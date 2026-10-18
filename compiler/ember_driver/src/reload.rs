//! Annex B in the driver: which functions a reloadable build reaches through thunks (`[HR-25]`),
//! and the build of a reloadable program — the shared runtime, the image, the host — or of the
//! next image of a running one (`ember build --reload`, `[BUD-2]`'s B2).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ember_build::{Layout, Profile, Toolchain};
use ember_codegen_c::ReloadImage;
use ember_span::{SourceMap, Span};

use crate::ReloadMode;

/// `@reloadable` (`Some(true)`) or `@noreload` (`Some(false)`) among `attrs`.
fn written(attrs: &[ember_ast::Attribute]) -> Option<bool> {
    attrs.iter().rev().find_map(|attr| match attr.path.as_slice() {
        [name] if name.name.is("reloadable") => Some(true),
        [name] if name.name.is("noreload") => Some(false),
        _ => None,
    })
}

fn has_attribute(attrs: &[ember_ast::Attribute], wanted: &str) -> bool {
    attrs.iter().any(|attr| attr.path.len() == 1 && attr.path[0].name.is(wanted))
}

/// The source text a span covers, hashed: what a `@noreload` body or a static's initialiser is,
/// for telling whether an edit changed it ([HR-10], [HR-17a]).
fn text_hash(map: &SourceMap, span: Span) -> u64 {
    let Some(file) = map.get(span.file) else { return 0 };
    let text = file.text.get(span.start as usize..span.end as usize).unwrap_or("");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// `[HR-25]` — what a reloadable build reaches through thunks: with `reload = "all"` or
/// `"bodies"`, every function of the package but those `@noreload` names; with `"opt-in"`, only
/// those `@reloadable` names. Either applies to a module (`#! module noreload`) or an item, the
/// item winning, and a method takes its type's. The standard library is not the package. A function
/// written `@inline` is inlined, so it is never reloadable (`[HR-9a]`).
pub fn plan(
    mode: ReloadMode,
    package: &str,
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
    bodies: &[ember_mir::Body],
) -> ReloadImage {
    // Each declared function's span start, with whether it is reloadable and its body's hash.
    let mut decided: BTreeMap<(ember_span::FileId, u32), (bool, u64)> = BTreeMap::new();
    let mut statics = BTreeMap::new();
    let mut files = BTreeSet::new();
    for loaded in modules {
        if loaded.path.first().is_some_and(|first| first == ember_branding::STD_PACKAGE) {
            continue;
        }
        files.insert(loaded.module.span.file);
        let module_default = loaded.module.directives.iter().rev().find_map(|directive| match &directive.value {
            ember_ast::DirectiveValue::ModuleAttribute(attr) => written(std::slice::from_ref(attr)),
            ember_ast::DirectiveValue::Text(_) => None,
        });
        let fallback = module_default.unwrap_or(mode != ReloadMode::OptIn);
        let decide = |attrs: &[ember_ast::Attribute], owner: Option<bool>| {
            !has_attribute(attrs, "inline") && written(attrs).or(owner).unwrap_or(fallback)
        };
        for item in &loaded.module.items {
            let owner = written(&item.attrs);
            let members = match &item.kind {
                ember_ast::ItemKind::Fn(decl) => {
                    if decl.body.is_some() && !decl.is_foreign_decl {
                        decided.insert((item.span.file, item.span.start), (decide(&item.attrs, None), text_hash(map, item.span)));
                    }
                    continue;
                }
                ember_ast::ItemKind::Static(decl) => {
                    if let Some(value) = &decl.value {
                        statics.insert(decl.name.name.to_string(), (text_hash(map, value.span), has_attribute(&item.attrs, "reinit_on_reload")));
                    }
                    continue;
                }
                ember_ast::ItemKind::Struct(decl) => &decl.members,
                ember_ast::ItemKind::Class(decl) => &decl.members,
                ember_ast::ItemKind::Enum(decl) => &decl.members,
                ember_ast::ItemKind::Extend(decl) => &decl.members,
                _ => continue,
            };
            for member in members {
                if let ember_ast::MemberKind::Fn(decl) = &member.kind
                    && decl.body.is_some()
                {
                    decided.insert((member.span.file, member.span.start), (decide(&member.attrs, owner), text_hash(map, member.span)));
                }
            }
        }
    }
    let mut reloadable = BTreeSet::new();
    let mut noreload = BTreeMap::new();
    for body in bodies {
        if body.is_lambda || body.is_abstract || body.is_extern_declaration || body.restrict_views || body.ffi_counted.is_some() {
            continue;
        }
        let Some(&(chosen, hash)) = decided.get(&(body.span.file, body.span.start)) else { continue };
        if chosen {
            reloadable.insert(body.symbol.clone());
        } else {
            noreload.insert(body.symbol.clone(), hash);
        }
    }
    ReloadImage {
        mode: match mode {
            ReloadMode::OptIn => 1,
            ReloadMode::All | ReloadMode::None => 2,
            ReloadMode::Bodies => 3,
        },
        // `[HR-19]` — the bodies tier needs no larger header; the others migrate instances.
        instances: mode != ReloadMode::Bodies,
        package: package.to_string(),
        reloadable,
        noreload,
        statics,
        files,
    }
}

/// The image file of number `n`: `<module>.image.<n>.dll` (or `.so`) beside the host.
fn image_path(bin: &Path, module: &str, n: u32) -> PathBuf {
    bin.join(format!("{module}.image.{n}.{}", ember_build::reload::image_extension()))
}

/// The file a running program's polls read for the newest image ([HR-31]).
pub fn announcement_path(bin: &Path, module: &str) -> PathBuf {
    bin.join(format!("{module}.reload"))
}

/// What a reloadable build made: the host to run, and the image it built.
pub struct Built {
    pub host: PathBuf,
    pub image: PathBuf,
}

/// Build a reloadable program: the shared runtime beside it ([HR-29]), its first image and its
/// host; or, with `next`, the next image of the program built before, announced to its polls.
#[allow(clippy::too_many_arguments)]
pub fn build(
    toolchain: &Toolchain,
    layout: &Layout,
    module: &str,
    c_path: &Path,
    runtime_source: &Path,
    includes: &[PathBuf],
    profile: Profile,
    image: &ReloadImage,
    next: bool,
    started: std::time::Instant,
) -> Result<Built, String> {
    let host = layout.bin.join(if cfg!(windows) { format!("{module}.exe") } else { module.to_string() });
    let runtime = ember_build::reload::shared_runtime(
        toolchain,
        runtime_source,
        includes,
        profile,
        image.instances,
        &ember_build::cache_root(),
        &layout.bin,
    )
    .map_err(|error| error.to_string())?;
    let number = if next {
        if !image_path(&layout.bin, module, 0).is_file() || !host.is_file() {
            return Err(format!(
                "there is no build of `{module}` to reload: run `ember build` first, then `ember build --reload` after each edit"
            ));
        }
        (1..).find(|n| !image_path(&layout.bin, module, *n).exists()).unwrap_or(1)
    } else {
        // A new program: its images start again at 0. One a running program still holds stays.
        if let Ok(entries) = std::fs::read_dir(&layout.bin) {
            let prefix = format!("{module}.image.");
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with(&prefix) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
        let _ = std::fs::remove_file(announcement_path(&layout.bin, module));
        0
    };
    let output = image_path(&layout.bin, module, number);
    ember_build::reload::link_image(
        toolchain,
        c_path,
        includes,
        profile,
        image.instances,
        &runtime,
        &ember_codegen_c::manifest_symbol(),
        &layout.obj,
        &output,
    )
    .map_err(|error| error.to_string())?;
    if next {
        // [HR-32] The build's own time travels with the announcement, for `ember_reload_stats`.
        let name = output.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let announcement = format!("{name}\n{}\n", started.elapsed().as_nanos());
        let path = announcement_path(&layout.bin, module);
        let staging = path.with_extension("reload.tmp");
        std::fs::write(&staging, announcement).map_err(|error| error.to_string())?;
        std::fs::rename(&staging, &path).map_err(|error| error.to_string())?;
    } else {
        ember_build::reload::link_host(toolchain, module, includes, profile, &runtime, &layout.obj, &host)
            .map_err(|error| error.to_string())?;
    }
    Ok(Built { host, image: output })
}
