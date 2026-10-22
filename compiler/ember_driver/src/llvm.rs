//! `[CG-LL-1]` — `--backend llvm` and `[build] backend = "llvm"`: the program as LLVM IR
//! (`ember_codegen_llvm`), which Clang compiles and links with the runtime as the C backend's C
//! is. The C backend stays the default.

use super::{ExitCode, Layout, LinkRequest, Options, Path, PathBuf, Profile, SourceMap, Toolchain, manifest_string};

/// Whether this build uses the LLVM backend: `--backend` first, else the manifest's
/// `[build] backend`.
pub(crate) fn selected(options: &Options, manifest: Option<&str>) -> Result<bool, String> {
    let named = options.backend.clone().or_else(|| manifest.and_then(|text| manifest_string(text, "build", "backend")));
    match named.as_deref() {
        None | Some("c") => Ok(false),
        Some("llvm") => Ok(true),
        Some(other) => Err(format!("`[build] backend` is \"c\" or \"llvm\", not `{other}`")),
    }
}

/// What installs Clang and LLD on this host.
fn clang_remedy() -> &'static str {
    if cfg!(windows) {
        "install LLVM (`winget install --id LLVM.LLVM`), which provides clang and lld"
    } else if cfg!(target_os = "macos") {
        "install the Xcode Command Line Tools (`xcode-select --install`), which provide clang"
    } else {
        "install clang and lld with the system's package manager (on Debian or Ubuntu, `sudo apt install clang lld`)"
    }
}

/// Clang, which compiles the IR and the runtime with it.
fn clang(options: &Options) -> Result<Toolchain, String> {
    let requested = options
        .cc
        .clone()
        .or_else(|| std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty()));
    if let Some(other) = requested.as_deref().filter(|cc| *cc != "clang" && *cc != "auto") {
        // The variable is a C backend's choice: only `--cc` is refused outright.
        if options.cc.is_some() {
            return Err(format!("the LLVM backend's IR is compiled by Clang, not `{other}` (`--cc clang`)"));
        }
    }
    Toolchain::detect(Some("clang")).map_err(|_| {
        format!("error[E9002]: the LLVM backend needs Clang, which was not found\n  = help: {}", clang_remedy())
    })
}

pub(crate) struct Build<'a> {
    pub mir: ember_mir::verify::VerifiedMir<'a>,
    pub map: &'a SourceMap,
    pub module_name: &'a str,
    pub has_main: bool,
    pub command: &'a str,
    pub library: bool,
    pub reload: bool,
    pub bodies: &'a [ember_mir::Body],
    pub input: &'a Path,
    pub manifest: Option<&'a str>,
}

fn profile(profile: Profile) -> ember_codegen_llvm::Profile {
    match profile {
        Profile::Debug => ember_codegen_llvm::Profile::Debug,
        Profile::Development => ember_codegen_llvm::Profile::Development,
        Profile::Release => ember_codegen_llvm::Profile::Release,
    }
}

/// The module emitted, written, compiled and linked; run for `ember run`.
pub(crate) fn build(request: Build<'_>, options: &Options, sink: &ember_diag::Sink) -> Result<ExitCode, String> {
    if request.library {
        return Err("the LLVM backend does not build a library package yet".to_string());
    }
    if request.reload {
        return Err("the LLVM backend does not build a reloadable program yet".to_string());
    }
    if request.command == "test" {
        return Err("the LLVM backend does not build tests yet (`ember test`)".to_string());
    }
    match options.emit.as_deref() {
        None | Some("llvm-ir") => {}
        Some(other) => return Err(format!("`--emit {other}` is the C backend's; the LLVM backend emits `llvm-ir`")),
    }
    let emit_options = ember_codegen_llvm::Options {
        module_name: request.module_name,
        has_main: request.has_main,
        leak_check: options.leak_check,
        profile: profile(options.profile),
        target: ember_codegen_llvm::Target::host(),
        triple: None,
    };
    let output = ember_codegen_llvm::emit(request.mir, request.map, &emit_options).map_err(|error| error.to_string())?;
    super::mark("LLVM IR generation", "all modules");
    if options.emit.as_deref() == Some("llvm-ir") {
        print!("{}", output.ir);
        return Ok(super::finish(sink, request.map, options));
    }
    if !request.has_main {
        return Err("this file declares no `main`, so there is nothing to run".to_string());
    }
    let target_dir = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target"));
    let layout = Layout::new(&target_dir, options.profile).map_err(|e| e.to_string())?;
    let ll_path = layout.c.join(format!("{}.ll", request.module_name));
    std::fs::write(&ll_path, &output.ir).map_err(|e| e.to_string())?;
    let toolchain = clang(options)?;
    let runtime = super::runtime_dir()?;
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let includes = vec![runtime.join("include")];
    let runtime_input = ember_build::runtime_object_for_executable(&toolchain, &runtime_source, &includes, options.profile, &ember_build::cache_root())
        .map_err(|e| e.to_string())?
        .unwrap_or(runtime_source);
    let exe_name = if cfg!(windows) { format!("{}.exe", request.module_name) } else { request.module_name.to_string() };
    let exe = layout.bin.join(&exe_name);
    // `[BLD-FFI-3]` — `[link] libs`, in the order written.
    let libraries: Vec<String> = request.manifest.and_then(|text| super::manifest_list(text, "link", "libs")).unwrap_or_default();
    // The exports stay though nothing in the program calls them.
    let mut retained: Vec<String> = request
        .bodies
        .iter()
        .filter(|body| body.abi.is_some() && !body.is_extern_declaration && !body.is_abstract)
        .map(|body| ember_branding::export_alias_symbol(&body.symbol).unwrap_or_else(|| body.symbol.clone()))
        .collect();
    retained.sort();
    retained.dedup();
    let _ = request.input;
    ember_build::compile_and_link_program(
        &toolchain,
        &LinkRequest {
            sources: &[ll_path, runtime_input],
            include_dirs: &includes,
            output: exe.clone(),
            profile: options.profile,
            obj_dir: layout.obj.clone(),
        },
        &retained,
        &libraries,
    )
    .map_err(|e| e.to_string())?;
    super::mark("LLVM compile and link", "all modules");
    super::report(sink, request.map, options);
    if request.command == "run" {
        return super::run_program(&exe, options);
    }
    eprintln!("built {}", exe.display());
    Ok(ExitCode::SUCCESS)
}
