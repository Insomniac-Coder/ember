//! Annex B in the driver: which functions a reloadable build reaches through thunks (`[HR-25]`),
//! and the build of a reloadable program — the shared runtime, the image, the host — or of the
//! next image of a running one (`ember build --reload`, `[BUD-2]`'s B2).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ember_build::{Layout, Profile, Toolchain};
use ember_codegen_c::ReloadImage;
use ember_diag::Sink;
use ember_span::{SourceMap, Span, Symbol};

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
/// item winning, and a method takes its type's. The standard library is not the package. `@inline`
/// does not change this: a reloadable function is never inlined, whatever `@inline` says (`[HR-9a]`,
/// `[CG-C-3a]`, ODR-119).
pub fn plan(
    mode: ReloadMode,
    package: &str,
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
    bodies: &[ember_mir::Body],
    additions: &Additions,
) -> ReloadImage {
    // Each declared function's span start, with whether it is reloadable and its body's hash.
    let mut decided: BTreeMap<(ember_span::FileId, u32), (bool, u64)> = BTreeMap::new();
    let mut statics = BTreeMap::new();
    let mut files = BTreeSet::new();
    let mut headers = BTreeMap::new();
    for loaded in modules {
        if let Some(foreign) = &loaded.foreign
            && let Some(hash) = &foreign.header_hash
        {
            headers.insert(foreign.header.clone(), hash.clone());
        }
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
            written(attrs).or(owner).unwrap_or(fallback)
        };
        for item in &loaded.module.items {
            // What the driver wrote for the reload is called directly by its own image.
            if additions.files.contains(&item.span.file) {
                continue;
            }
            let owner = written(&item.attrs);
            let members = match &item.kind {
                ember_ast::ItemKind::Fn(decl) => {
                    if decl.body.is_some() && !decl.is_foreign_decl {
                        decided.insert((item.span.file, item.span.start), (decide(&item.attrs, None), text_hash(map, item.span)));
                    }
                    continue;
                }
                ember_ast::ItemKind::Static(decl) => {
                    // Named as the checker names it: bare in the root module, by its module's path
                    // elsewhere.
                    if let Some(value) = &decl.value {
                        let name = if loaded.path.is_empty() {
                            decl.name.name.to_string()
                        } else {
                            format!("{}.{}", loaded.path.join("."), decl.name.name)
                        };
                        statics.insert(
                            name,
                            (text_hash(map, value.span), text_hash(map, decl.ty.span), has_attribute(&item.attrs, "reinit_on_reload")),
                        );
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
        files: files.into_iter().chain(additions.files.iter().copied()).collect(),
        defaults: additions.defaults.clone(),
        renamed: additions.renamed.clone(),
        migrations: additions.migrations.clone(),
        headers,
    }
}

/// `[LNT-5]` — `L2005`: a `@noreload` function calling a reloadable one inside a loop pays the
/// thunk's load and indirect jump on every turn, where the hot code it was kept out of reload for
/// wants none ([HR-10]).
pub fn lint_noreload_loops(bodies: &[ember_mir::Body], image: &ReloadImage, sink: &mut ember_diag::Sink) {
    use ember_mir::{FuncRef, Terminator};
    let successors = |terminator: &Terminator| -> Vec<usize> {
        match terminator {
            Terminator::Goto(bb) => vec![bb.0 as usize],
            Terminator::SwitchInt { targets, otherwise, .. } => {
                targets.iter().map(|(_, bb)| bb.0 as usize).chain(std::iter::once(otherwise.0 as usize)).collect()
            }
            Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
            Terminator::Return | Terminator::Unreachable => Vec::new(),
        }
    };
    for body in bodies.iter().filter(|body| image.noreload.contains_key(&body.symbol)) {
        for (index, block) in body.blocks.iter().enumerate() {
            let Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } = &block.terminator else { continue };
            if !image.reloadable.contains(symbol) {
                continue;
            }
            // ponytail: a walk from each call's block, quadratic in a body's blocks; a loop
            // forest would serve if one ever is large.
            let mut seen = vec![false; body.blocks.len()];
            let mut pending = successors(&block.terminator);
            let mut in_loop = false;
            while let Some(at) = pending.pop() {
                if at == index {
                    in_loop = true;
                    break;
                }
                if at < seen.len() && !std::mem::replace(&mut seen[at], true) {
                    pending.extend(successors(&body.blocks[at].terminator));
                }
            }
            if !in_loop {
                continue;
            }
            let callee = bodies.iter().find(|other| other.symbol == *symbol).map_or(symbol.as_str(), |other| other.name.as_str());
            sink.emit(
                ember_diag::Diagnostic::lint(
                    ember_diag::codes::L2005,
                    block.terminator_span,
                    format!("`@noreload` function `{}` calls reloadable `{callee}` inside a loop", body.name),
                )
                .note("each turn pays the thunk's load and indirect jump [HR-9], in code kept out of reload for its speed [HR-10]")
                .help(format!("mark `{callee}` `@noreload` too, or move the loop into a reloadable function")),
            );
        }
    }
}

/// The image file of number `n`: `<module>.image.<n>.dll` (or `.so`) beside the host.
fn image_path(bin: &Path, module: &str, n: u32) -> PathBuf {
    bin.join(format!("{module}.image.{n}.{}", ember_build::reload::image_extension()))
}

/// `[HR-18b]` — ask the running program to plan `image` against itself, applying nothing, and
/// wait for its answer: the plan's verdict and its report. Its next poll or checkpoint answers.
pub fn explain(bin: &Path, module: &str, image: &Path) -> Result<(bool, String), String> {
    let request = announcement_path(bin, module).with_extension("reload.explain");
    let answer = announcement_path(bin, module).with_extension("reload.explained");
    let _ = std::fs::remove_file(&answer);
    let name = image.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let staging = request.with_extension("tmp");
    std::fs::write(&staging, format!("{name}\n")).map_err(|error| error.to_string())?;
    std::fs::rename(&staging, &request).map_err(|error| error.to_string())?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        if let Ok(text) = std::fs::read_to_string(&answer)
            && let Some(rest) = text.strip_prefix(&format!("{name}\n"))
            && let Some((verdict, report)) = rest.split_once('\n')
        {
            let _ = std::fs::remove_file(&request);
            return Ok((verdict == "apply", report.to_string()));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let _ = std::fs::remove_file(&request);
    Err(format!("no running `{module}` answered within ten seconds: `--explain` predicts against the running program, at its next poll or checkpoint"))
}

/// The file a running program's polls read for the newest image ([HR-31]).
pub fn announcement_path(bin: &Path, module: &str) -> PathBuf {
    bin.join(format!("{module}.reload"))
}

/// Why a reloadable build stopped.
pub enum Failure {
    /// `[HR-12a]` — the next image's object header is not the running program's.
    Header { running: u32, built: u32 },
    Other(String),
}

impl From<String> for Failure {
    fn from(message: String) -> Failure {
        Failure::Other(message)
    }
}

/// The file recording the object header a program was built with, beside it.
fn header_path(bin: &Path, module: &str) -> PathBuf {
    bin.join(format!("{module}.header"))
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
    units: &[PathBuf],
    runtime_source: &Path,
    includes: &[PathBuf],
    profile: Profile,
    image: &ReloadImage,
    relaxed_units: &[(ember_types::FpMode, String)],
    foreign: &(Vec<PathBuf>, Vec<String>),
    next: bool,
    explain: bool,
    started: std::time::Instant,
    jobs: Option<usize>,
) -> Result<Built, Failure> {
    let host = layout.bin.join(if cfg!(windows) { format!("{module}.exe") } else { module.to_string() });
    let header: u32 = if image.instances { 40 } else { 24 };
    if next {
        let running = std::fs::read_to_string(header_path(&layout.bin, module)).ok().and_then(|text| text.trim().parse().ok());
        if let Some(running) = running.filter(|running| *running != header) {
            return Err(Failure::Header { running, built: header });
        }
    }
    let runtime = ember_build::reload::shared_runtime(
        toolchain,
        runtime_source,
        includes,
        profile,
        Some(image.instances),
        &ember_build::cache_root(),
        &layout.bin,
    )
    .map_err(|error| error.to_string())?;
    let number = if next {
        if !image_path(&layout.bin, module, 0).is_file() || !host.is_file() {
            return Err(Failure::Other(format!(
                "there is no build of `{module}` to reload: run `ember build` first, then `ember build --reload` after each edit"
            )));
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
        std::fs::write(header_path(&layout.bin, module), format!("{header}\n")).map_err(|error| error.to_string())?;
        0
    };
    // `[HR-18b]` — an image to explain is never announced, and has a name of its own.
    let output = if explain {
        let extension = ember_build::reload::image_extension();
        (1..)
            .map(|n| layout.bin.join(format!("{module}.explain.{n}.{extension}")))
            .find(|path| !path.exists())
            .unwrap_or_else(|| layout.bin.join(format!("{module}.explain.{extension}")))
    } else {
        image_path(&layout.bin, module, number)
    };
    // `[CG-C-11]` — each relaxed unit compiled with its own flags into the image.
    let mut relaxed = Vec::new();
    let object_ext = if matches!(toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
    let stem = output.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_else(|| module.to_string());
    for (mode, source) in relaxed_units {
        let unit = layout.c.join(format!("{stem}.{}.c", mode.name()));
        std::fs::write(&unit, source).map_err(|error| error.to_string())?;
        let object = layout.obj.join(format!("{stem}.{}.{object_ext}", mode.name()));
        ember_build::compile_relaxed_library_object(toolchain, &unit, includes, &object, profile, *mode == ember_types::FpMode::Fast)
            .map_err(|error| error.to_string())?;
        relaxed.push(object);
    }
    // `[FFI-29]`, `[BLD-FFI-3]` — the imports' shims and thunks are the image's, and their libraries.
    relaxed.extend(foreign.0.iter().cloned());
    // `[BLD-4]` — the image's units (the program's own and, one per module, `debug`'s and
    // `development`'s) compiled through Ninja: a unit whose text did not change is not compiled again.
    let defines = ember_build::reload::reload_defines(image.instances);
    let manifest = ember_codegen_c::manifest_symbol();
    ember_build::ninja::build(
        toolchain,
        &ember_build::ninja::Program {
            units,
            objects: &relaxed,
            include_dirs: includes,
            output: output.clone(),
            profile,
            dir: layout.obj.clone(),
            jobs,
            image: Some(ember_build::ninja::Image { defines: &defines, link_with: &runtime.link_with, manifest: &manifest }),
        },
        &[],
        &foreign.1,
    )
    .map_err(|error| error.to_string())?;
    if next && !explain {
        // [HR-32] The build's own time travels with the announcement, for `ember_reload_stats`.
        let name = output.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let announcement = format!("{name}\n{}\n", started.elapsed().as_nanos());
        let path = announcement_path(&layout.bin, module);
        let staging = path.with_extension("reload.tmp");
        std::fs::write(&staging, announcement).map_err(|error| error.to_string())?;
        std::fs::rename(&staging, &path).map_err(|error| error.to_string())?;
    } else if !next {
        ember_build::reload::link_host(toolchain, module, includes, profile, &runtime, &layout.obj, &host)
            .map_err(|error| error.to_string())?;
    }
    Ok(Built { host, image: output })
}

/// What the driver writes into a reloadable program before it is checked (`inject`), for the
/// backend's schemas and the next build's `Old<Name>` structs.
#[derive(Default)]
pub struct Additions {
    /// `[HR-14]` — each field with a declared default: its type's name and its own, with the
    /// function computing the default.
    pub defaults: BTreeMap<(String, String), String>,
    /// `[HR-14]` — `@renamed_from("old")`: the type's name and the field's or variant's, with the
    /// old name.
    pub renamed: BTreeMap<(String, String), String>,
    /// `[HR-16]` — each type declaring `migrate_from`: its wrapper and its `Old<Name>` struct.
    pub migrations: BTreeMap<String, (String, String)>,
    /// The files the injected source was parsed from.
    pub files: BTreeSet<ember_span::FileId>,
    /// This build's schema, by type: each field's name and written type, which the next build's
    /// `Old<Name>` structs are made from.
    pub schema: serde_json::Map<String, serde_json::Value>,
    /// `[HR-14]` — the functions written to give a field declared without a default its type's
    /// `Default.default()`, by span: the checker builds one only where the type is `Default`.
    pub if_default: std::collections::HashSet<Span>,
    /// `[HR-39]`, `[HR-43]` — each `migrate_from`, by its declaration's span, with its type's name
    /// and, when it is `@allow_reload_terminate`, the attribute's span.
    pub migrate_fns: Vec<(Span, String, Option<Span>)>,
}

/// A written type whose `Default` is all zero bytes, which the runtime writes itself
/// (`EMBER_RL_FIELD_ZERO`): a number, `bool`, `String` or an `Array`.
fn zero_default(ty: &ember_ast::TypeExpr) -> bool {
    const ZERO: [&str; 19] = [
        "int", "uint", "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "isize", "usize", "f32", "f64", "bool",
        "String", "Array",
    ];
    matches!(&ty.kind, ember_ast::TypeKind::Path { segments, .. } if segments.len() == 1 && ZERO.contains(&segments[0].name.as_str()))
}

/// A name the injected source gives a type's helper: its module-local name, made of the type's.
fn helper(prefix: &str, ty: &str, rest: &str) -> String {
    let ty: String = ty.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    if rest.is_empty() { format!("{prefix}_{ty}") } else { format!("{prefix}_{ty}_{rest}") }
}

/// The text of an attribute's one string argument: `@renamed_from("old")`'s `old`.
fn string_argument(attrs: &[ember_ast::Attribute], wanted: &str) -> Option<String> {
    attrs.iter().filter(|attr| attr.path.len() == 1 && attr.path[0].name.is(wanted)).find_map(|attr| match attr.args.first() {
        Some(ember_ast::AttrArg::Expr(ember_ast::Expr { kind: ember_ast::ExprKind::Lit(ember_ast::Literal::Str(text)), .. })) => {
            Some(text.to_string())
        }
        _ => None,
    })
}

/// `OldSelf` in a written type, for `Old<Name>` ([HR-16]).
fn rename_old_self(ty: &mut ember_ast::TypeExpr, old: Symbol) {
    match &mut ty.kind {
        ember_ast::TypeKind::Path { segments, args } => {
            if segments.len() == 1 && segments[0].name.is("OldSelf") {
                segments[0].name = old;
            }
            for arg in args {
                if let ember_ast::GenericArg::Type(inner) | ember_ast::GenericArg::Assoc { ty: inner, .. } = arg {
                    rename_old_self(inner, old);
                }
            }
        }
        ember_ast::TypeKind::Ref { inner, .. } | ember_ast::TypeKind::Ptr { inner, .. } => rename_old_self(inner, old),
        ember_ast::TypeKind::Tuple(items) => items.iter_mut().for_each(|item| rename_old_self(item, old)),
        _ => {}
    }
}

/// `[HR-35]` — `migrate_from` may not contain an explicit panic: `panic`, `unwrap`, `expect` or
/// `assert` is `E2225`, whose help names the fallible form. Found among the tokens of its text, so a
/// word in a string or a comment is not one.
fn check_migrate_from_panics(map: &SourceMap, span: Span, sink: &mut Sink) {
    use ember_lexer::{Punct, TokenKind};
    let Some(file) = map.get(span.file) else { return };
    let Some(text) = file.text.get(span.start as usize..span.end as usize) else { return };
    let mut scratch = Sink::new();
    let tokens = ember_lexer::lex(span.file, text, &mut scratch).tokens;
    for (at, token) in tokens.iter().enumerate() {
        let Some(word) = token.ident() else { continue };
        let after_dot = at > 0 && tokens[at - 1].kind == TokenKind::Punct(Punct::Dot);
        let called = tokens.get(at + 1).is_some_and(|next| next.kind == TokenKind::Punct(Punct::LParen));
        if !called {
            continue;
        }
        let help = match (word.as_str(), after_dot) {
            ("unwrap" | "expect", true) => "use `?` to return the error, or `match` it and return `Err(ReloadError.Migration(…))`",
            ("panic", false) => "return `Err(ReloadError.Migration(type_name, instance, message))` instead",
            ("assert" | "debug_assert", false) => "test it with `if` and return `Err(ReloadError.Migration(…))`",
            _ => continue,
        };
        let at_span = Span::new(span.file, span.start + token.span.start, span.start + token.span.end);
        sink.emit(
            ember_diag::Diagnostic::error(ember_diag::codes::E2225, at_span, format!("`migrate_from` may not panic: `{word}` would end the process"))
                .help(help.to_string())
                .note("a reload that cannot migrate a value is refused with a report; it never aborts [HR-35]"),
        );
    }
}

/// `[HR-39]` — a `migrate_from` that can reach a C++ function called `noexcept` (derived from its
/// declaration, called directly, or asserted by an overlay, `[FFI-24]`, `[FFI-24b]`), directly or
/// through the program's own functions it calls, could end the process with a throw instead of
/// refusing the reload: `E2225`, naming the function and the path to it, unless the `migrate_from`
/// is `@allow_reload_terminate` (`[HR-43]`). A C++ function that catches returns its throw as a
/// `CppError`, which `?` makes `ReloadError.Foreign`.
pub fn check_migrate_from_terminates(
    bodies: &[ember_mir::Body],
    modules: &[ember_typeck::LoadedModule],
    additions: &Additions,
    sink: &mut Sink,
) {
    let cpp_files: BTreeSet<ember_span::FileId> = modules
        .iter()
        .filter(|loaded| loaded.foreign.as_ref().is_some_and(|foreign| foreign.cpp.is_some()))
        .map(|loaded| loaded.module.span.file)
        .collect();
    if cpp_files.is_empty() {
        return;
    }
    let standard: BTreeSet<ember_span::FileId> = modules
        .iter()
        .filter(|loaded| loaded.path.first().is_some_and(|first| first == ember_branding::STD_PACKAGE))
        .map(|loaded| loaded.module.span.file)
        .collect();
    let by_symbol: std::collections::HashMap<&str, &ember_mir::Body> = bodies.iter().map(|body| (body.symbol.as_str(), body)).collect();
    for (span, type_name, allowed) in &additions.migrate_fns {
        if allowed.is_some() {
            continue;
        }
        let Some(root) = bodies.iter().find(|body| body.span.file == span.file && body.span.start == span.start) else { continue };
        // Each body reached, with the call in `migrate_from` it was reached through and the
        // functions between.
        let mut queue: std::collections::VecDeque<(&ember_mir::Body, Option<Span>, Vec<String>)> = std::collections::VecDeque::from([(root, None, Vec::new())]);
        let mut seen: BTreeSet<&str> = BTreeSet::from([root.symbol.as_str()]);
        let mut reported: BTreeSet<&str> = BTreeSet::new();
        while let Some((body, through, path)) = queue.pop_front() {
            for block in &body.blocks {
                let ember_mir::Terminator::Call { func: ember_mir::FuncRef::Direct { symbol, .. }, .. } = &block.terminator else { continue };
                let Some(callee) = by_symbol.get(symbol.as_str()).copied() else { continue };
                let site = through.unwrap_or(block.terminator_span);
                if let Some(mode) = crate::cpp_call_mode(callee, &by_symbol, &cpp_files) {
                    if mode.starts_with("noexcept") && reported.insert(callee.symbol.as_str()) {
                        let via = if path.is_empty() { String::new() } else { format!(" through `{}`", path.join("` → `")) };
                        sink.emit(
                            ember_diag::Diagnostic::error(
                                ember_diag::codes::E2225,
                                site,
                                format!("`{type_name}.migrate_from` may not call `{}`{via}, a C++ function called {mode}: a throw from it ends the process", callee.name),
                            )
                            .help("call a function that catches (its throw comes back as a `CppError`, which `?` makes `ReloadError.Foreign`), or mark `migrate_from` `@allow_reload_terminate` to state that this migration may end the process rather than refuse [HR-43]")
                            .note("a reload that cannot migrate a value is refused with a report; it never aborts [HR-35], [HR-39]"),
                        );
                    }
                    continue;
                }
                if standard.contains(&callee.span.file) || cpp_files.contains(&callee.span.file) || !seen.insert(callee.symbol.as_str()) {
                    continue;
                }
                let mut next = path.clone();
                next.push(callee.name.clone());
                queue.push_back((callee, Some(site), next));
            }
        }
    }
}

/// Annex B — before a reloadable program is checked, the driver writes into each of the package's
/// modules what its reload needs: for each field with a declared default, a function computing it
/// (`[HR-14]`'s "initialised to that default"); for each type declaring `migrate_from`, the struct
/// `Old<Name>` of the previous build's fields (`[HR-16]`, from `previous`, this build's own when
/// there is none), `OldSelf` in its signature renamed to it, and a wrapper the backend calls,
/// turning a `ReloadError` into its report. It also gathers `@renamed_from` and this build's schema.
pub fn inject(
    modules: &mut [ember_typeck::LoadedModule],
    map: &mut SourceMap,
    sink: &mut Sink,
    previous: Option<&serde_json::Value>,
) -> Additions {
    let mut additions = Additions::default();
    for loaded in modules.iter_mut() {
        if loaded.path.first().is_some_and(|first| first == ember_branding::STD_PACKAGE) {
            continue;
        }
        let prefix = if loaded.path.is_empty() { String::new() } else { format!("{}.", loaded.path.join(".")) };
        let mut source = String::new();
        let mut if_default: BTreeMap<String, ember_ast::TypeExpr> = BTreeMap::new();
        let snippet = |span: Span| map.get(span.file).and_then(|file| file.text.get(span.start as usize..span.end as usize)).unwrap_or("").to_string();
        for item in std::sync::Arc::make_mut(&mut loaded.module).items.iter_mut() {
            let (ident, members, generic) = match &mut item.kind {
                ember_ast::ItemKind::Struct(decl) => (decl.name.name, &mut decl.members, !decl.generics.is_empty()),
                ember_ast::ItemKind::Class(decl) => (decl.name.name, &mut decl.members, !decl.generics.is_empty()),
                ember_ast::ItemKind::Enum(decl) => {
                    let name = format!("{prefix}{}", decl.name.name);
                    for variant in &decl.variants {
                        if let Some(old) = string_argument(&variant.attrs, "renamed_from") {
                            additions.renamed.insert((name.clone(), variant.name.name.to_string()), old);
                        }
                        // `[HR-14]` — a payload field renamed, or added with its type `Default`, as a
                        // type's field.
                        for (index, field) in variant.fields.iter().enumerate() {
                            let field_name = field.name.map_or_else(|| format!("_{index}"), |name| name.name.to_string());
                            if let Some(old) = string_argument(&field.attrs, "renamed_from") {
                                additions.renamed.insert((name.clone(), format!("{}.{field_name}", variant.name.name)), old);
                            }
                            if !decl.generics.is_empty() || zero_default(&field.ty) {
                                continue;
                            }
                            let function = helper("__reload_default", &format!("{}_{}", decl.name.name, variant.name.name), &field_name);
                            source.push_str(&format!("fn {function}() -> {}:\n    return 0\n\n", snippet(field.ty.span)));
                            if_default.insert(function.clone(), field.ty.clone());
                            additions.defaults.insert((name.clone(), format!("{}.{field_name}", variant.name.name)), function);
                        }
                    }
                    continue;
                }
                _ => continue,
            };
            if generic {
                continue;
            }
            let name = format!("{prefix}{ident}");
            let mut fields = Vec::new();
            for member in members.iter() {
                let ember_ast::MemberKind::Field(field) = &member.kind else { continue };
                let field_name = field.name.name.to_string();
                let ty = snippet(field.ty.span);
                fields.push(serde_json::json!([field_name, ty]));
                if let Some(old) = string_argument(&member.attrs, "renamed_from") {
                    additions.renamed.insert((name.clone(), field_name.clone()), old);
                }
                if let Some(default) = &field.default {
                    let function = helper("__reload_default", ident.as_str(), &field_name);
                    source.push_str(&format!("fn {function}() -> {ty}:\n    return {}\n\n", snippet(default.span)));
                    additions.defaults.insert((name.clone(), field_name), function);
                } else if !zero_default(&field.ty) {
                    // `[HR-14]` — added, its type `Default`: `std.core.default_of[T]()`, written
                    // below as the derives write it; the checker drops it where `T` is not `Default`.
                    let function = helper("__reload_default", ident.as_str(), &field_name);
                    source.push_str(&format!("fn {function}() -> {ty}:\n    return 0\n\n"));
                    if_default.insert(function.clone(), field.ty.clone());
                    additions.defaults.insert((name.clone(), field_name), function);
                }
            }
            additions.schema.insert(name.clone(), serde_json::Value::Array(fields.clone()));
            let Some(member) = members.iter_mut().find(|member| matches!(&member.kind, ember_ast::MemberKind::Fn(decl) if decl.name.name.is("migrate_from"))) else {
                continue;
            };
            check_migrate_from_panics(map, member.span, sink);
            let terminate = member.attrs.iter().find(|attr| attr.path.len() == 1 && attr.path[0].name.is("allow_reload_terminate")).map(|attr| attr.span);
            additions.migrate_fns.push((member.span, name.clone(), terminate));
            let ember_ast::MemberKind::Fn(decl) = &mut member.kind else { continue };
            // `Old<Name>` under a helper's name, which no source spells: it is visible only as
            // `migrate_from`'s `OldSelf`, and a type of the package named `Old<Name>` is its own.
            let old_name = helper("__reload_Old", ident.as_str(), "");
            let old_symbol = Symbol::intern(&old_name);
            for param in &mut decl.params {
                if let ember_ast::ParamKind::Named { ty, .. } = &mut param.kind {
                    rename_old_self(ty, old_symbol);
                }
            }
            // The error type as the signature spells it: `Result[Self, ReloadError]`'s second argument.
            let error = decl.ret.as_ref().and_then(|ret| match &ret.kind {
                ember_ast::TypeKind::Path { args, .. } if args.len() == 2 => match &args[1] {
                    ember_ast::GenericArg::Type(error) => Some(snippet(error.span)),
                    _ => None,
                },
                _ => None,
            });
            let Some(error) = error else { continue };
            let old_fields: Vec<(String, String)> = previous
                .and_then(|previous| previous.get(&name))
                .and_then(serde_json::Value::as_array)
                .unwrap_or(&fields)
                .iter()
                .filter_map(|pair| Some((pair.get(0)?.as_str()?.to_string(), pair.get(1)?.as_str()?.to_string())))
                .collect();
            source.push_str(&format!("struct {old_name}:\n"));
            if old_fields.is_empty() {
                source.push_str("    pass\n");
            }
            for (field, ty) in &old_fields {
                source.push_str(&format!("    {field}: {ty}\n"));
            }
            let wrapper = helper("__reload_migrate", ident.as_str(), "");
            source.push_str(&format!(
                "\nfn {wrapper}(old: ref {old_name}, mut report: String) -> Option[{ident}]:\n    match {ident}.migrate_from(old):\n        Ok(value):\n            return Some(value)\n        Err(error):\n            match error:\n                \
                 {error}.SchemaRefused(type_name, reason):\n                    report.push_str(f\"SchemaRefused({{type_name}}, {{reason}})\")\n                \
                 {error}.Migration(type_name, instance, message):\n                    report.push_str(f\"Migration({{type_name}}, {{instance}}, {{message}})\")\n                \
                 {error}.Allocation:\n                    report.push_str(\"Allocation\")\n                \
                 {error}.Foreign(error):\n                    report.push_str(f\"Foreign({{error}})\")\n                \
                 {error}.Timeout:\n                    report.push_str(\"Timeout\")\n            return None\n\n"
            ));
            additions.migrations.insert(name.clone(), (wrapper, format!("{prefix}{old_name}")));
        }
        if source.is_empty() {
            continue;
        }
        let shown = if loaded.path.is_empty() { "the root module".to_string() } else { loaded.path.join(".") };
        let file = map.add(PathBuf::from(format!("<reload helpers of {shown}>")), source.clone());
        additions.files.insert(file);
        let lexed = ember_lexer::lex(file, &source, sink);
        let mut parsed = ember_parser::parse(file, &source, lexed.tokens, sink);
        for item in &mut parsed.items {
            let ember_ast::ItemKind::Fn(decl) = &mut item.kind else { continue };
            let Some(ty) = if_default.get(decl.name.name.as_str()) else { continue };
            let Some(ember_ast::Stmt { kind: ember_ast::StmtKind::Expr(returned), .. }) = decl.body.as_mut().and_then(|body| body.stmts.first_mut()) else {
                continue;
            };
            let ember_ast::ExprKind::Jump(ember_ast::Jump::Return(Some(value))) = &mut returned.kind else { continue };
            let span = value.span;
            let expr = |kind| ember_ast::Expr { id: ember_ast::NodeId::DUMMY, kind, span };
            let callee = expr(ember_ast::ExprKind::IndexOrInstantiate {
                base: Box::new(expr(ember_ast::ExprKind::Path { segments: vec![ember_ast::Ident { name: Symbol::intern("std.core.default_of"), span }] })),
                args: vec![ember_ast::TypeOrExpr::Type(ty.clone())],
            });
            **value = expr(ember_ast::ExprKind::Call { callee: Box::new(callee), args: Vec::new() });
            additions.if_default.insert(item.span);
        }
        std::sync::Arc::make_mut(&mut loaded.module).items.extend(parsed.items);
    }
    additions
}
