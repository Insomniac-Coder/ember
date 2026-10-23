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

/// Whether this compilation's backend is LLVM, recorded as it starts (`mark_selected`) for the MIR
/// passes that tune code for the compiler that optimises it (`tuned_for_msvc`).
static SELECTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn mark_selected(llvm: bool) {
    SELECTED.store(llvm, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn is_selected() -> bool {
    SELECTED.load(std::sync::atomic::Ordering::Relaxed)
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
    /// `[FFI-28]` — a library package (`staticlib`, `cdylib`), its exports' names under
    /// `package_name`; a `cdylib`'s runtime its own or `[FFI-31c]` the process's (`shared_runtime`).
    pub library: bool,
    pub cdylib: bool,
    pub package_name: &'a str,
    pub shared_runtime: bool,
    /// Annex B — a reloadable program's image, as the driver decides it, and what it wrote into the
    /// program (the next build's `Old<Name>` structs are made from its schema).
    pub reload: Option<&'a ember_codegen_c::ReloadImage>,
    pub additions: Option<&'a super::reload::Additions>,
    pub started: std::time::Instant,
    /// `ember inspect`'s tables (`super::write_inspect`), which no backend changes.
    pub inspect: [String; 2],
    pub bodies: &'a [ember_mir::Body],
    pub input: &'a Path,
    pub manifest: Option<&'a str>,
    /// Each module, an `import c`'s with its shim and C sources.
    pub modules: &'a [ember_typeck::LoadedModule],
    /// `[TST-3]` — for `ember test`, the package's tests in the runner's order.
    pub tests: &'a [&'a ember_hir::TestFn],
}

fn profile(profile: Profile) -> ember_codegen_llvm::Profile {
    match profile {
        Profile::Debug => ember_codegen_llvm::Profile::Debug,
        Profile::Development => ember_codegen_llvm::Profile::Development,
        Profile::Release => ember_codegen_llvm::Profile::Release,
    }
}

/// `[BLD-FFI-3]` — `[link] libs`, in the order written, a path the package's; `[FFI-29]`,
/// `[BLD-FFI-1]` — each `import c`'s shim and C sources, compiled by the `[c]` section's compiler
/// (Clang by default) into objects of their own (`shared_library`: for a `cdylib`), then its
/// libraries.
fn link_inputs(request: &Build<'_>, toolchain: &Toolchain, layout: &Layout, profile: Profile, shared_library: bool) -> Result<(Vec<String>, Vec<PathBuf>), String> {
    let package_dir = super::nearest_manifest_path(request.input).and_then(|path| path.parent().map(Path::to_path_buf));
    let mut libraries: Vec<String> = request
        .manifest
        .and_then(|text| super::manifest_list(text, "link", "libs"))
        .unwrap_or_default()
        .into_iter()
        .map(|library| match &package_dir {
            Some(dir) if ember_build::cdylib::is_library_path(&library) && Path::new(&library).is_relative() => {
                dir.join(&library).to_string_lossy().into_owned()
            }
            _ => library,
        })
        .collect();
    let c_toolchain = match request.manifest.and_then(|text| manifest_string(text, "c", "compiler")) {
        Some(compiler) => Toolchain::detect(Some(&compiler)).map_err(|e| e.to_string())?,
        None => toolchain.clone(),
    };
    let (foreign_objects, foreign_libraries) =
        super::cimport::build_objects(request.modules, &c_toolchain, toolchain, &layout.c, &layout.obj, profile, shared_library)?;
    libraries.extend(foreign_libraries);
    Ok((libraries, foreign_objects))
}

/// The module emitted, written, compiled and linked; run for `ember run`. A reloadable program's
/// module is its image, which the shared runtime loads beside the host (Annex B).
pub(crate) fn build(request: Build<'_>, options: &Options, sink: &mut ember_diag::Sink) -> Result<ExitCode, String> {
    let testing = request.command == "test";
    match options.emit.as_deref() {
        None | Some("llvm-ir") => {}
        Some(other) => return Err(format!("`--emit {other}` is the C backend's; the LLVM backend emits `llvm-ir`")),
    }
    // `[FFI-26]`, `[BLD-FFI-5a]` — a library's headers are the C backend's, from the same type plan;
    // this module defines what they declare.
    let interface = request.library.then(|| {
        ember_codegen_c::library_interface(request.mir, request.map, request.module_name, request.package_name, request.cdylib, false)
    });
    let emit_options = ember_codegen_llvm::Options {
        module_name: request.module_name,
        has_main: request.has_main,
        // `[FFI-31]` — a shared library reports its leaks at shutdown in `debug`.
        leak_check: options.leak_check || (request.cdylib && options.profile == Profile::Debug),
        profile: profile(options.profile),
        target: ember_codegen_llvm::Target::host(),
        triple: None,
        tests: testing.then(|| request.tests.iter().map(|test| test.symbol.clone()).collect()),
        reload: request.reload,
        module: request.cdylib,
        package: request.library.then_some(request.package_name),
        classes: interface.as_ref().map_or(&[], |interface| interface.classes.as_slice()),
    };
    let output = ember_codegen_llvm::emit(request.mir, request.map, &emit_options).map_err(|error| error.to_string())?;
    super::mark("LLVM IR generation", "all modules");
    if options.emit.as_deref() == Some("llvm-ir") {
        print!("{}", output.ir);
        if let Some(shim) = &output.shim {
            print!("\n; the runtime's inline functions, called through this C:\n");
            for line in shim.lines() {
                println!("; {line}");
            }
        }
        return Ok(super::finish(sink, request.map, options));
    }
    if !request.has_main && !testing && !request.library {
        return Err("this file declares no `main`, so there is nothing to run".to_string());
    }
    let target_dir = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target"));
    let layout = Layout::new(&target_dir, options.profile).map_err(|e| e.to_string())?;
    super::write_inspect(&layout, request.module_name, &request.inspect)?;
    let ll_path = layout.c.join(format!("{}.ll", request.module_name));
    std::fs::write(&ll_path, &output.ir).map_err(|e| e.to_string())?;
    let mut sources = vec![ll_path];
    // The runtime's inline functions the IR calls, through their one-line C wrappers.
    if let Some(shim) = &output.shim {
        let shim_path = layout.c.join(format!("{}.rt.c", request.module_name));
        std::fs::write(&shim_path, shim).map_err(|e| e.to_string())?;
        sources.push(shim_path);
    }
    let toolchain = clang(options)?;
    // `[BLD-6]` — the build record: the IR goes through the same Clang link, ThinLTO's included (a
    // hot-reload image without it).
    super::write_build_record(&layout, request.module_name, options.profile, &toolchain, ember_build::Link::Here, request.reload.is_some())?;
    // Annex B — the image: the IR and its C, compiled with the reload definitions, as a shared
    // library beside the shared runtime and the host.
    if let Some(image) = request.reload {
        let program = super::ReloadProgram {
            toolchain: &toolchain,
            layout: &layout,
            module_name: request.module_name,
            units: &sources,
            relaxed_units: &[],
            image,
            additions: request.additions,
            modules: request.modules,
            manifest: request.manifest,
            started: request.started,
        };
        return super::build_reload_program(program, request.command, options, sink, request.map);
    }
    if let Some(interface) = interface {
        let header = interface.header?;
        let (libraries, foreign_objects) = link_inputs(&request, &toolchain, &layout, options.profile, request.cdylib)?;
        let library = super::Library {
            toolchain: &toolchain,
            layout: &layout,
            package_name: request.package_name,
            module_name: request.module_name,
            cdylib: request.cdylib,
            shared_runtime: request.shared_runtime,
            units: &sources,
            objects: &foreign_objects,
            libraries: &libraries,
            exports: if request.cdylib { super::cdylib_exports(request.bodies, request.mir.types()) } else { Vec::new() },
            header: &header,
        };
        return super::build_library(library, options, sink, request.map);
    }
    let runtime = super::runtime_dir()?;
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let includes = vec![runtime.join("include")];
    let runtime_input = ember_build::runtime_object_for_executable(&toolchain, &runtime_source, &includes, options.profile, &ember_build::cache_root())
        .map_err(|e| e.to_string())?
        .unwrap_or(runtime_source);
    let exe_name = if cfg!(windows) { format!("{}.exe", request.module_name) } else { request.module_name.to_string() };
    let exe = layout.bin.join(&exe_name);
    let (libraries, foreign_objects) = link_inputs(&request, &toolchain, &layout, options.profile, false)?;
    // The exports stay though nothing in the program calls them.
    let mut retained: Vec<String> = request
        .bodies
        .iter()
        .filter(|body| body.abi.is_some() && !body.is_extern_declaration && !body.is_abstract)
        .map(|body| ember_branding::export_alias_symbol(&body.symbol).unwrap_or_else(|| body.symbol.clone()))
        .collect();
    retained.sort();
    retained.dedup();
    sources.push(runtime_input);
    sources.extend(foreign_objects);
    ember_build::compile_and_link_program(
        &toolchain,
        &LinkRequest {
            sources: &sources,
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
    if testing {
        return super::test_runner::run_tests(&exe, request.tests, options, None, request.map);
    }
    if request.command == "run" {
        return super::run_program(&exe, options);
    }
    eprintln!("built {}", exe.display());
    Ok(ExitCode::SUCCESS)
}
