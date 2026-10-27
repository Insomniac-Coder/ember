//! `[BLD-4]` (ODR-244, ODR-258) — a program's C compiled and linked through a generated Ninja file.
//!
//! Each C unit is compiled to an object of its own and the objects are linked into the program, by
//! a `build.ninja` the build writes beside the objects and Ninja runs: a unit whose text, flags and
//! included files are as they were is not compiled again (Ninja keeps each edge's command and each
//! object's included files), and the units Ninja compiles run at once. A build writes a unit's file
//! only when its text changed, so its time stamp says when it last did. The flags are the one-call
//! build's ([`crate::compile_and_link_program`]), split between the compile and the link; a
//! program built before from the same inputs is copied from the global cache as there.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{
    BuildError, CachedProgram, LinkRequest, Lto, Link, Profile, Toolchain, build_lto, compiler_command, executable_section_flags,
    executable_section_gc_supported, folder_name_flags, gcc_flags, gnu_flags, inherited_gnu_lto, is_clang_cl, layout_flags,
    lto_compile_flags, lto_link_flags, msvc_flags, natvis_beside, program_cache, warnings_as_errors,
};

/// A program built through Ninja: the C units compiled with the program's flags, the objects
/// linked as they are (the runtime's, the relaxed units', each import's), the include folders,
/// the program, and the folder its objects and its Ninja file go in.
pub struct Program<'a> {
    pub units: &'a [PathBuf],
    pub objects: &'a [PathBuf],
    pub include_dirs: &'a [PathBuf],
    pub output: PathBuf,
    pub profile: Profile,
    pub dir: PathBuf,
    /// `[BLD-7]` — `-j`: the compiles Ninja runs at once (its own default otherwise).
    pub jobs: Option<usize>,
    /// Annex B — the units make a reloadable program's image, not a program.
    pub image: Option<Image<'a>>,
}

/// Annex B (`[BLD-4]`) — a reloadable program's image: its units compiled with the reload
/// definitions as a shared library's objects, and linked into one against the shared runtime
/// (`link_with`), keeping its manifest (`[HR-8]`), as `crate::reload::link_image` links one unit.
#[derive(Clone, Copy)]
pub struct Image<'a> {
    pub defines: &'a [String],
    pub link_with: &'a Path,
    pub manifest: &'a str,
}

/// Write `text` to `path` unless it holds it already: an unchanged unit keeps its time stamp, so
/// Ninja does not compile it again. Whether it was written.
pub fn write_if_changed(path: &Path, text: &str) -> std::io::Result<bool> {
    if std::fs::read(path).is_ok_and(|bytes| bytes == text.as_bytes()) {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(true)
}

/// `[BLD-4]` — build `program` through Ninja (`crate::ninja`), linking `libraries` after its own
/// objects and keeping `retained` though nothing calls them (as `compile_and_link_program`).
pub fn build(toolchain: &Toolchain, program: &Program<'_>, retained: &[String], libraries: &[String]) -> Result<(), BuildError> {
    std::fs::create_dir_all(&program.dir)?;
    // Ninja runs in the objects' folder: every path it is given is absolute, and the build's own
    // files are spelled from that folder (`from_dir`).
    let absolute = |path: &PathBuf| std::path::absolute(path).unwrap_or_else(|_| path.clone());
    let units: Vec<PathBuf> = program.units.iter().map(absolute).collect();
    let linked: Vec<PathBuf> = program.objects.iter().map(absolute).collect();
    let include_dirs: Vec<PathBuf> = program.include_dirs.iter().map(absolute).collect();
    // An image links against the shared runtime by its path; the image names the runtime by its
    // soname, so the absolute path reaches no output.
    let link_with: Option<PathBuf> = program.image.map(|image| absolute(&image.link_with.to_path_buf()));
    let program = &Program {
        units: &units,
        objects: &linked,
        include_dirs: &include_dirs,
        output: absolute(&program.output),
        profile: program.profile,
        dir: absolute(&program.dir),
        jobs: program.jobs,
        image: program.image.map(|image| Image { link_with: link_with.as_deref().unwrap_or(image.link_with), ..image }),
    };
    let lto = build_lto(toolchain, program.profile, Link::Here);
    let objects: Vec<PathBuf> = program.units.iter().map(|unit| object_of(toolchain, &program.dir, unit)).collect();
    let compile = match &program.image {
        Some(image) => image_compile_args(toolchain, program, image, lto),
        None => compile_args(toolchain, program, lto),
    };
    let link = match &program.image {
        Some(image) => image_link_args(toolchain, program, image, lto, libraries),
        None => link_args(toolchain, program, lto, retained, libraries),
    };
    // The program cache's key: the commands without their outputs, the units and objects by what
    // they hold (`CachedProgram::find`). An image is the next one of a running program: none.
    let cache = if libraries.is_empty() && program.image.is_none() { program_cache() } else { None };
    let inputs: Vec<PathBuf> = program.units.iter().chain(program.objects).cloned().collect();
    let request = LinkRequest { sources: &inputs, include_dirs: program.include_dirs, output: program.output.clone(), profile: program.profile, obj_dir: program.dir.clone() };
    let mut key = compiler_command(toolchain);
    key.arg("ninja-program").args(&compile).arg("link").args(&link).args(&inputs);
    let cached = cache.as_deref().and_then(|dir| CachedProgram::find(dir, toolchain, &key, &request));
    if let Some(cached) = &cached
        && cached.serve(&program.output)
    {
        return Ok(());
    }
    let ninja = crate::ninja().map_err(BuildError::NoNinja)?;
    let msvc = matches!(toolchain, Toolchain::Msvc { .. });
    let compiler = compiler_command(toolchain);
    let program_name = compiler.get_program().to_os_string();
    let prefix: Vec<OsString> = compiler.get_args().map(|arg| arg.to_os_string()).collect();
    let mut text = String::from("# Generated by the Ember compiler. Do not edit. [BLD-4]\nninja_required_version = 1.10\n\n");
    if msvc {
        text.push_str(&format!("msvc_deps_prefix = {}\n\n", escape_value(&msvc_deps_prefix(toolchain))));
    }
    let compile_command = |input: &str, output: &str| -> String {
        let mut words: Vec<String> = vec![quote(&program_name.to_string_lossy())];
        words.extend(prefix.iter().chain(&compile).map(|arg| quote(&arg.to_string_lossy())));
        if msvc {
            words.extend(["/showIncludes".to_string(), "/c".to_string(), input.to_string(), format!("/Fo{output}")]);
        } else {
            words.extend(["-MD".to_string(), "-MF".to_string(), format!("{output}.d"), "-c".to_string(), input.to_string(), "-o".to_string(), output.to_string()]);
        }
        words.join(" ")
    };
    text.push_str("rule cc\n");
    text.push_str(&format!("  command = {}\n", escape_value(&compile_command("$in", "$out"))));
    if msvc {
        text.push_str("  deps = msvc\n");
    } else {
        text.push_str("  depfile = $out.d\n  deps = gcc\n");
    }
    text.push_str("  description = C $in\n\n");
    // The LLVM backend's units (`[CG-LL-1]`): its IR, compiled by Clang with the program's flags and
    // the ones IR needs (`crate::ir_flags`); an IR unit includes nothing, so it has no dependency file.
    if program.units.iter().any(|unit| is_ir(unit)) {
        let mut words: Vec<String> = vec![quote(&program_name.to_string_lossy())];
        words.extend(prefix.iter().chain(&compile).map(|arg| quote(&arg.to_string_lossy())));
        words.extend(crate::ir_flags(&[Path::new("unit.ll")]).iter().map(|flag| flag.to_string()));
        words.extend(["-c".to_string(), "$in".to_string(), "-o".to_string(), "$out".to_string()]);
        text.push_str("rule ll\n");
        text.push_str(&format!("  command = {}\n", escape_value(&words.join(" "))));
        text.push_str("  description = IR $in\n\n");
    }
    // The link's inputs and flags in a response file: four hundred objects are more than a command
    // line holds.
    let mut link_words: Vec<String> = vec![quote(&program_name.to_string_lossy())];
    link_words.extend(prefix.iter().map(|arg| quote(&arg.to_string_lossy())));
    link_words.push("@$out.rsp".to_string());
    text.push_str("rule link\n");
    text.push_str(&format!("  command = {}\n", escape_value(&link_words.join(" "))));
    text.push_str("  rspfile = $out.rsp\n");
    text.push_str("  rspfile_content = $rsp\n");
    text.push_str("  description = link $out\n\n");
    for (unit, object) in program.units.iter().zip(&objects) {
        let rule = if is_ir(unit) { "ll" } else { "cc" };
        text.push_str(&format!("build {}: {rule} {}\n", escape_path(&from_dir(&program.dir, object)), escape_path(&from_dir(&program.dir, unit))));
    }
    let linked: Vec<PathBuf> = objects.iter().chain(program.objects).map(|path| from_dir(&program.dir, path)).collect();
    let mut rsp: Vec<OsString> = linked.iter().map(|path| path.as_os_str().to_os_string()).collect();
    rsp.extend(link.iter().cloned());
    if program.image.is_none() {
        rsp.extend(output_args(toolchain, program));
    }
    let output = from_dir(&program.dir, &program.output);
    text.push_str(&format!(
        "build {}: link {}\n  rsp = {}\n",
        escape_path(&output),
        linked.iter().map(|path| escape_path(path)).collect::<Vec<_>>().join(" "),
        escape_value(&rsp_words(&rsp, !msvc))
    ));
    text.push_str(&format!("default {}\n", escape_path(&output)));
    let file = program.dir.join("build.ninja");
    write_if_changed(&file, &text)?;
    let mut run = Command::new(&ninja);
    run.arg("-C").arg(&program.dir).arg("-f").arg("build.ninja").arg("--quiet");
    if let Some(jobs) = program.jobs {
        run.arg("-j").arg(jobs.to_string());
    }
    if let Toolchain::Msvc { env, .. } = toolchain {
        run.envs(env);
    }
    let started = std::time::SystemTime::now();
    let output = crate::output_started(&mut run)?;
    if !output.status.success() {
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        return Err(BuildError::CompilerFailed { command: format!("{run:?}"), output: text });
    }
    if let Some(cached) = cached {
        cached.store_files(&request, included_files(&ninja, &program.dir), started);
    }
    Ok(())
}

/// The files the units included, as Ninja recorded them (`ninja -t deps`).
fn included_files(ninja: &Path, dir: &Path) -> Vec<PathBuf> {
    let Ok(output) = Command::new(ninja).arg("-C").arg(dir).args(["-f", "build.ninja", "-t", "deps"]).output() else { return Vec::new() };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with("    "))
        .map(|line| {
            let path = PathBuf::from(line.trim());
            if path.is_absolute() { path } else { dir.join(path) }
        })
        .collect()
}

/// A unit's object: beside the Ninja file, named after the unit.
fn object_of(toolchain: &Toolchain, dir: &Path, unit: &Path) -> PathBuf {
    let extension = if matches!(toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
    // The unit's whole name with the object's extension: `p.m.geom.c` is `p.m.geom.obj`.
    dir.join(format!("{}.{extension}", unit.file_stem().unwrap_or_default().to_string_lossy()))
}

/// The program's compile flags, without the unit and the object: the one-call build's.
fn compile_args(toolchain: &Toolchain, program: &Program<'_>, lto: Lto) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::new();
    let (layout_compile, _) = layout_flags(toolchain, crate::env_layout());
    match toolchain {
        Toolchain::Msvc { cl, .. } => {
            // Each object holds its own debug information (`/Z7`): units compiled at once share no
            // program database; the link writes the program's.
            args.extend(msvc_flags(program.profile).into_iter().map(|flag| if flag == "/Zi" { "/Z7" } else { flag }).map(OsString::from));
            args.extend(lto_compile_flags(toolchain, lto, Link::Here).iter().map(OsString::from));
            if warnings_as_errors() {
                args.push("/WX".into());
            }
            if is_clang_cl(cl) {
                args.push("/Brepro".into());
                if program.profile != Profile::Release {
                    args.push("/clang:-gno-codeview-command-line".into());
                    args.push("/clang:-fdebug-compilation-dir=.".into());
                }
                let folders: Vec<PathBuf> = program.units.iter().cloned().chain([program.dir.join("object")]).collect();
                args.extend(folder_name_flags(&folders, program.include_dirs).into_iter().map(|flag| OsString::from(format!("/clang:{flag}"))));
            }
            args.extend(program.include_dirs.iter().map(|dir| OsString::from(format!("/I{}", dir.display()))));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            args.extend(gnu_flags(program.profile).into_iter().chain(gcc_flags(toolchain, program.profile).iter().copied()).map(OsString::from));
            match inherited_gnu_lto() {
                Some(inherited) if !inherited.is_empty() => args.push(inherited.into()),
                Some(_) => {}
                None => args.extend(lto_compile_flags(toolchain, lto, Link::Here).iter().map(OsString::from)),
            }
            args.extend(layout_compile.iter().map(OsString::from));
            if warnings_as_errors() {
                args.push("-Werror".into());
            }
            args.extend(folder_name_flags(program.units, program.include_dirs).into_iter().map(OsString::from));
            args.extend(executable_section_flags(toolchain, true).iter().map(OsString::from));
            for dir in program.include_dirs {
                args.push("-I".into());
                args.push(dir.into());
            }
        }
    }
    args
}

/// Annex B — an image's compile flags, without the unit and the object: a shared library's object's
/// (`crate::reload::compile_library_object`), with the reload definitions and the profile's
/// link-time optimisation, as the program built without reload has (`[HR-9]`): what it inlines
/// across units are the functions that are not reloadable (the standard library's, `@noreload`
/// ones, the imports' C); a reloadable body it never reaches, every call to one loading its target
/// from the image's table (`[HR-6]`, `[HR-9a]`).
fn image_compile_args(toolchain: &Toolchain, program: &Program<'_>, image: &Image<'_>, lto: Lto) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::new();
    match toolchain {
        Toolchain::Msvc { .. } => {
            args.extend(msvc_flags(program.profile).into_iter().map(|flag| if flag == "/Zi" { "/Z7" } else { flag }).map(OsString::from));
            args.extend(lto_compile_flags(toolchain, lto, Link::Here).iter().map(OsString::from));
            args.extend(image.defines.iter().map(|define| OsString::from(format!("/D{define}"))));
            args.extend(program.include_dirs.iter().map(|dir| OsString::from(format!("/I{}", dir.display()))));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            args.extend(gnu_flags(program.profile).into_iter().chain(gcc_flags(toolchain, program.profile).iter().copied()).map(OsString::from));
            args.extend(lto_compile_flags(toolchain, lto, Link::Here).iter().map(OsString::from));
            if !cfg!(windows) {
                // An image exports nothing: the runtime finds its manifest by section ([HR-8]).
                args.extend(["-fPIC", "-fvisibility=hidden"].map(OsString::from));
            }
            args.extend(image.defines.iter().map(|define| OsString::from(format!("-D{define}"))));
            for dir in program.include_dirs {
                args.push("-I".into());
                args.push(dir.into());
            }
        }
    }
    args
}

/// Annex B — an image's link after its objects, its output included: a shared library against the
/// shared runtime, keeping the manifest.
fn image_link_args(toolchain: &Toolchain, program: &Program<'_>, image: &Image<'_>, lto: Lto, libraries: &[String]) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::new();
    match toolchain {
        Toolchain::Msvc { .. } => {
            args.extend(["/nologo", "/LD", crate::msvc_runtime_switch()].map(OsString::from));
            // clang-cl links with LLVM's linker, as the program's link does: it reads the objects
            // link-time optimisation leaves.
            if matches!(toolchain, Toolchain::Msvc { cl, .. } if is_clang_cl(cl)) {
                args.push("-fuse-ld=lld".into());
            }
            args.push(image.link_with.into());
            args.extend(libraries.iter().map(|library| OsString::from(crate::cdylib::windows_library(library))));
            args.push(format!("/Fe{}", program.output.display()).into());
            args.push("/link".into());
            args.push(format!("/INCLUDE:{}", image.manifest).into());
            args.extend(lto_link_flags(toolchain, lto).iter().map(OsString::from));
            if program.profile != Profile::Release {
                args.push("/DEBUG".into());
                args.push(format!("/PDB:{}", program.output.with_extension("pdb").display()).into());
            }
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            args.push("-shared".into());
            args.push(image.link_with.into());
            args.extend(["-o".into(), program.output.clone().into_os_string()]);
            args.extend(lto_link_flags(toolchain, lto).iter().map(OsString::from));
            args.extend(libraries.iter().map(|library| OsString::from(crate::cdylib::gnu_library(library))));
            if cfg!(windows) {
                args.push(format!("-Wl,/INCLUDE:{}", image.manifest).into());
            } else {
                args.extend(["-lm", "-pthread"].map(OsString::from));
            }
        }
    }
    args
}

/// The program's link flags after its objects: the one-call build's.
fn link_args(toolchain: &Toolchain, program: &Program<'_>, lto: Lto, retained: &[String], libraries: &[String]) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::new();
    let (_, layout_link) = layout_flags(toolchain, crate::env_layout());
    match toolchain {
        Toolchain::Msvc { cl, .. } => {
            args.extend(msvc_flags(program.profile).into_iter().map(OsString::from));
            args.extend(libraries.iter().map(|library| OsString::from(crate::cdylib::windows_library(library))));
            if is_clang_cl(cl) {
                args.push("-fuse-ld=lld".into());
            }
            args.extend(["/link", "/Brepro", "/INCREMENTAL:NO"].map(OsString::from));
            args.extend(lto_link_flags(toolchain, lto).iter().map(OsString::from));
            args.extend(retained.iter().map(|symbol| OsString::from(format!("/INCLUDE:{symbol}"))));
            if program.profile != Profile::Release {
                args.push("/PDBALTPATH:%_PDB%".into());
                if is_clang_cl(cl) {
                    args.push("/pdbsourcepath:.".into());
                }
            }
            if program.profile == Profile::Development {
                args.extend(["/OPT:REF", "/OPT:ICF"].map(OsString::from));
            }
            args.extend(layout_link.iter().map(OsString::from));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            args.extend(gnu_flags(program.profile).into_iter().chain(gcc_flags(toolchain, program.profile).iter().copied()).map(OsString::from));
            if toolchain.is_bundled() {
                args.push("-fuse-ld=lld".into());
            }
            match inherited_gnu_lto() {
                Some(inherited) if !inherited.is_empty() => {
                    args.push(inherited.into());
                    if cfg!(windows) && matches!(toolchain, Toolchain::Clang(_)) {
                        args.push("-fuse-ld=lld".into());
                    }
                }
                Some(_) => {}
                None => args.extend(lto_link_flags(toolchain, lto).iter().map(OsString::from)),
            }
            args.extend(libraries.iter().map(|library| OsString::from(crate::cdylib::gnu_library(library))));
            if !cfg!(windows) {
                args.extend(["-lm", "-pthread"].map(OsString::from));
            }
            let section_gc = executable_section_gc_supported(toolchain);
            if section_gc {
                args.extend(["-Xlinker", "--gc-sections"].map(OsString::from));
            }
            for symbol in retained {
                match (section_gc, cfg!(windows)) {
                    (true, _) => args.extend([OsString::from("-Xlinker"), format!("--require-defined={symbol}").into()]),
                    (false, true) => args.extend([OsString::from("-Xlinker"), format!("/INCLUDE:{symbol}").into()]),
                    (false, false) => args.extend([OsString::from("-Xlinker"), "-u".into(), "-Xlinker".into(), symbol.into()]),
                }
            }
            if cfg!(windows) {
                args.extend(["-Wl,/Brepro", "-Wl,/INCREMENTAL:NO"].map(OsString::from));
                if program.profile != Profile::Release {
                    args.push("-Wl,/PDBALTPATH:%_PDB%".into());
                }
                if program.profile == Profile::Development {
                    args.extend(["-Wl,/OPT:REF", "-Wl,/OPT:ICF"].map(OsString::from));
                }
            }
        }
    }
    args
}

/// The link's flags that name where its outputs go, which the program cache's key leaves out: the
/// program, and the visualisers it puts in the program database (keyed by their content).
fn output_args(toolchain: &Toolchain, program: &Program<'_>) -> Vec<OsString> {
    let request = LinkRequest { sources: &[], include_dirs: &[], output: program.output.clone(), profile: program.profile, obj_dir: program.dir.clone() };
    let natvis = natvis_beside(&request).map(|natvis| from_dir(&program.dir, &natvis));
    let output = from_dir(&program.dir, &program.output);
    match toolchain {
        // After `/link`: the linker's own.
        Toolchain::Msvc { .. } => {
            let mut out = vec![OsString::from(format!("/OUT:{}", output.display()))];
            out.extend(natvis.map(|natvis| OsString::from(format!("/NATVIS:{}", natvis.display()))));
            out
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            let mut out = vec![OsString::from("-o"), output.into()];
            if cfg!(windows) {
                out.extend(natvis.map(|natvis| OsString::from(format!("-Wl,/NATVIS:{}", natvis.display()))));
            }
            out
        }
    }
}

/// What MSVC writes before each file `/showIncludes` names, in the compiler's own language: Ninja
/// reads the included files by it (`deps = msvc`). Asked of the compiler once and kept in the
/// global cache by the compiler's file.
fn msvc_deps_prefix(toolchain: &Toolchain) -> String {
    const ENGLISH: &str = "Note: including file:";
    let Some(compiler) = crate::compiler_file(toolchain) else { return ENGLISH.to_string() };
    let stamp = compiler.metadata().map(|meta| format!("{} {}", meta.len(), crate::modified_nanos(&meta))).unwrap_or_default();
    let key = blake3::hash(format!("msvc-deps-prefix v1\0{}\0{stamp}", compiler.display()).as_bytes()).to_hex();
    let file = crate::cache_root().join("msvc").join(format!("{key}.prefix"));
    if let Ok(prefix) = std::fs::read_to_string(&file) {
        return prefix;
    }
    let dir = std::env::temp_dir().join(format!("{}-deps-prefix-{}", ember_branding::CLI_NAME, std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let probe = dir.join("probe.c");
    let found = std::fs::write(&probe, "#include <stddef.h>\n").ok().and_then(|()| {
        let mut command = compiler_command(toolchain);
        command.args(["/nologo", "/showIncludes", "/Zs"]).arg(&probe).current_dir(&dir);
        let output = command.output().ok()?;
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.lines().find_map(|line| {
            let at = line.find("stddef.h")?;
            let before = &line[..at];
            // The path is absolute: the prefix ends before its drive (`C:\`) or `\\server`.
            let start = before.rfind(":\\").map(|colon| colon.saturating_sub(1)).or_else(|| before.find("\\\\"))?;
            Some(line[..start].trim_end().to_string())
        })
    });
    let _ = std::fs::remove_dir_all(&dir);
    match found {
        Some(prefix) if !prefix.is_empty() => {
            crate::write_atomically(&file, &prefix);
            prefix
        }
        _ => ENGLISH.to_string(),
    }
}

/// A word of a command line, quoted for the shell Ninja runs it in when it needs to be: on Windows
/// Ninja starts the program itself (the C runtime's rules), elsewhere through `/bin/sh -c`.
fn quote(word: &str) -> String {
    if !word.is_empty() && !word.contains(|c: char| c.is_whitespace() || c == '"' || (!cfg!(windows) && "'\\$`!*?[]{}()<>|&;#~".contains(c))) {
        return word.to_string();
    }
    if cfg!(windows) {
        // The C runtime's rules: backslashes before a quote are doubled, a quote escaped.
        let mut out = String::from("\"");
        let mut backslashes = 0;
        for c in word.chars() {
            match c {
                '\\' => backslashes += 1,
                '"' => {
                    out.push_str(&"\\".repeat(backslashes * 2 + 1));
                    out.push('"');
                    backslashes = 0;
                }
                _ => {
                    out.push_str(&"\\".repeat(backslashes));
                    out.push(c);
                    backslashes = 0;
                }
            }
        }
        out.push_str(&"\\".repeat(backslashes * 2));
        out.push('"');
        out
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

/// The link's flags as a response file holds them: one per line, quoted as a command line's.
/// The words of a response file. cl and clang-cl read one as Windows reads a command line; clang
/// and gcc read one as GNU tools do, on Windows too, where a backslash escapes the next character:
/// each word is quoted and its backslashes and quotes escaped.
fn rsp_words(args: &[OsString], gnu: bool) -> String {
    let word = |arg: &OsString| {
        let text = arg.to_string_lossy();
        if gnu { format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\"")) } else { quote(&text) }
    };
    args.iter().map(word).collect::<Vec<_>>().join(" ")
}

/// A value in a Ninja file: `$` is written `$$`.
fn escape_value(text: &str) -> String {
    text.replace('$', "$$").replace("$$in", "$in").replace("$$out", "$out")
}

/// A path in a `build` line: `$`, a space and `:` escaped.
fn escape_path(path: &Path) -> String {
    path.to_string_lossy().replace('$', "$$").replace(' ', "$ ").replace(':', "$:")
}

/// `[BLD-13]` — a file of the build's own spelled from the Ninja folder, where Ninja runs the
/// commands: an object by its name, a unit, the program and its visualisers from the folder above
/// (`..\c\main.c`, `..\bin\main.exe`). The objects' debug information, a link-time optimised
/// object's module (its source file's name, which ThinLTO hashes into the names it promotes) and
/// the program database name them as the commands spell them, so the same inputs built in another
/// folder are the same bytes. Any other path (the runtime's cached objects) stays as it is.
fn from_dir(dir: &Path, path: &Path) -> PathBuf {
    if let Ok(name) = path.strip_prefix(dir) {
        return name.to_path_buf();
    }
    match dir.parent().and_then(|above| path.strip_prefix(above).ok()) {
        Some(rest) => Path::new("..").join(rest),
        None => path.to_path_buf(),
    }
}

/// Whether a unit is the LLVM backend's IR (`.ll`) rather than C.
fn is_ir(unit: &Path) -> bool {
    unit.extension().is_some_and(|extension| extension == "ll")
}
