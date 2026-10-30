//! `[CG-LL-1]` — the LLVM backend: the program as LLVM IR (`ember_codegen_llvm`), which Clang compiles
//! and links with the runtime as the C backend's C is. It is the default since it passed the
//! conformance and the performance suites (2026-10-30); `--backend c` and `[build] backend = "c"`
//! choose the C backend, which stays supported.

use super::{ExitCode, Layout, Options, Path, PathBuf, Profile, SourceMap, Toolchain, manifest_string};

/// Whether this build uses the LLVM backend: `--backend` first, then the manifest's `[build]
/// backend`, then `EMBER_BACKEND` (`ember_branding::backend_var`); LLVM when none names one.
pub(crate) fn selected(options: &Options, manifest: Option<&str>) -> Result<bool, String> {
    let named = options
        .backend
        .clone()
        .or_else(|| manifest.and_then(|text| manifest_string(text, "build", "backend")))
        .or_else(|| std::env::var(ember_branding::backend_var()).ok().filter(|backend| !backend.is_empty()));
    match named.as_deref() {
        Some("c") => Ok(false),
        None | Some("llvm") => Ok(true),
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
pub(crate) fn clang(options: &Options) -> Result<Toolchain, String> {
    let requested = options
        .cc
        .clone()
        .or_else(|| std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty()));
    if let Some(other) = requested.as_deref().filter(|cc| *cc != "clang" && *cc != "auto") {
        // The variable is a C backend's choice: only `--cc` is refused outright.
        if options.cc.is_some() {
            return Err(format!(
                "the LLVM backend's IR is compiled by Clang, not `{other}`: build with `--backend c --cc {other}` for the C backend compiled by `{other}`, or `--cc clang`"
            ));
        }
    }
    Toolchain::detect(Some("clang")).map_err(|_| {
        format!(
            "error[E9002]: the LLVM backend, the default, needs Clang, which was not found\n  = help: {}\n  = help: or build with the C backend, which any C compiler compiles: `--backend c`, or `backend = \"c\"` under `[build]` in `{}`",
            clang_remedy(),
            ember_branding::MANIFEST
        )
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
    /// `[CG-LL-3]` (ODR-274) — the profile-guided optimisation asked for.
    pub pgo: Option<super::PgoAsk>,
    /// Annex B — a reloadable program's image, as the driver decides it, and what it wrote into the
    /// program (the next build's `Old<Name>` structs are made from its schema).
    pub reload: Option<&'a ember_codegen_c::ReloadImage>,
    pub additions: Option<&'a super::reload::Additions>,
    pub started: std::time::Instant,
    /// `ember inspect`'s tables (`super::write_inspect`), which no backend changes.
    pub inspect: [String; 2],
    /// `[SIMD-2]` — each `@simd` loop's verdict, for `--emit-optimization-report`.
    pub simd_report: &'a [String],
    /// `[CLI-10]` — `--report=engine`'s report, all but Clang's word on the loops, which the build adds.
    pub engine: Option<super::EngineReport>,
    /// `[BLD-4]` — the program as a unit per module (`super::module_units`), or as one.
    pub units: Option<Vec<(String, std::collections::BTreeSet<String>)>>,
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

/// `[CG-LL-3]` (ODR-274) — the profile `--pgo-use` names: a merged `.profdata` as it is; a raw
/// `.profraw`, or a folder of them (`--pgo-generate`'s `pgo` folder), merged by `llvm-profdata`
/// into this build's own `pgo/<module>.profdata` first.
fn profile_data(toolchain: &Toolchain, recorded: &Path, layout: &Layout, module_name: &str) -> Result<PathBuf, String> {
    if recorded.extension().is_some_and(|ext| ext == "profdata") {
        return if recorded.is_file() {
            std::path::absolute(recorded).map_err(|e| e.to_string())
        } else {
            Err(format!("no recorded profile at {} [CG-LL-3]", recorded.display()))
        };
    }
    let raw: Vec<PathBuf> = if recorded.is_dir() {
        let mut found: Vec<PathBuf> = std::fs::read_dir(recorded)
            .map_err(|e| e.to_string())?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "profraw"))
            .collect();
        found.sort();
        found
    } else if recorded.is_file() {
        vec![recorded.to_path_buf()]
    } else {
        Vec::new()
    };
    if raw.is_empty() {
        return Err(format!(
            "no recorded profile at {}: run the program built with `--pgo-generate` first, which writes its raw profiles there [CG-LL-3]",
            recorded.display()
        ));
    }
    let merged = std::path::absolute(layout.root.join("pgo").join(format!("{module_name}.profdata"))).map_err(|e| e.to_string())?;
    ember_build::merge_profiles(toolchain, &raw, &merged).map_err(|e| e.to_string())?;
    Ok(merged)
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
pub(crate) fn build(mut request: Build<'_>, options: &Options, sink: &mut ember_diag::Sink) -> Result<ExitCode, String> {
    let testing = request.command == "test";
    match options.emit.as_deref() {
        None | Some("llvm-ir") => {}
        Some(other) => {
            return Err(format!("`--emit {other}` is the C backend's (`--backend c --emit {other}`); the LLVM backend emits `llvm-ir`"));
        }
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
        instrument_ffi: testing && options.ffi_evidence.is_some(),
        units: request.units.as_deref(),
    };
    let output = ember_codegen_llvm::emit(request.mir, request.map, &emit_options).map_err(|error| error.to_string())?;
    super::mark("LLVM IR generation", "all modules");
    if options.optimization_report {
        super::print_optimization_report(&output.optimization_report, request.simd_report);
    }
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
    // `[CG-C-9]` — the debugger visualisers beside the program, as the C backend's: a `.natvis`
    // (which the link also puts in the program database), GDB's auto-loaded `<program>-gdb.py`, and
    // an LLDB script.
    let exe_name = if cfg!(windows) { format!("{}.exe", request.module_name) } else { request.module_name.to_string() };
    std::fs::write(layout.bin.join(format!("{}.natvis", request.module_name)), &output.visualisers.natvis).map_err(|e| e.to_string())?;
    std::fs::write(layout.bin.join(format!("{exe_name}-gdb.py")), &output.visualisers.gdb).map_err(|e| e.to_string())?;
    std::fs::write(layout.bin.join(format!("{}.lldb.py", request.module_name)), &output.visualisers.lldb).map_err(|e| e.to_string())?;
    // `[BLD-4]` — each unit written only when its text changed, so the build compiles again only
    // those that did; a module's unit from an earlier build that this one has not is removed.
    let ll_path = layout.c.join(format!("{}.ll", request.module_name));
    ember_build::ninja::write_if_changed(&ll_path, &output.ir).map_err(|e| e.to_string())?;
    let mut sources = vec![ll_path];
    let prefix = format!("{}.m.", request.module_name);
    for (name, ir) in &output.units {
        let path = layout.c.join(format!("{prefix}{name}.ll"));
        ember_build::ninja::write_if_changed(&path, ir).map_err(|e| e.to_string())?;
        sources.push(path);
    }
    if let Ok(entries) = std::fs::read_dir(&layout.c) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(&prefix) && name.ends_with(".ll") && !sources.contains(&path) {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    // The runtime's inline functions the IR calls, through their one-line C wrappers.
    if let Some(shim) = &output.shim {
        let shim_path = layout.c.join(format!("{}.rt.c", request.module_name));
        ember_build::ninja::write_if_changed(&shim_path, shim).map_err(|e| e.to_string())?;
        sources.push(shim_path);
    }
    // `[FFI-37]` — an instrumented test's counting, the C backend's unit (it defines the C library's
    // entry points it intercepts off Windows).
    if let Some(instrument) = &output.ffi_instrument {
        let unit = layout.c.join(format!("{}.ffi.c", request.module_name));
        ember_build::ninja::write_if_changed(&unit, &instrument.source).map_err(|e| e.to_string())?;
        sources.push(unit);
    }
    let toolchain = clang(options)?;
    ember_build::set_pgo(match &request.pgo {
        None => None,
        Some(_) if request.library || request.reload.is_some() || testing => {
            return Err("profile-guided optimisation builds a program ([CG-LL-3]), not a library, a reloadable image or a test binary".to_string());
        }
        // Raw profiles in the profile's own folder, where the program writes them as it ends.
        Some(super::PgoAsk::Generate) => {
            let dir = layout.root.join("pgo");
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            Some(ember_build::Pgo::Generate(std::path::absolute(&dir).map_err(|e| e.to_string())?))
        }
        Some(super::PgoAsk::Use(recorded)) => Some(ember_build::Pgo::Use(profile_data(&toolchain, recorded, &layout, request.module_name)?)),
    });
    // `[BLD-6]` — the build record: the IR goes through the same Clang link, ThinLTO's included (a
    // hot-reload image without it).
    super::write_build_record(&layout, request.module_name, options.profile, &toolchain, ember_build::Link::Here, request.reload.is_some())?;
    if options.build_id {
        println!("{}", super::build_id(request.map, options.profile, "llvm", &toolchain, &super::runtime_dir()?)?);
    }
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
    let exe = layout.bin.join(&exe_name);
    let (libraries, foreign_objects) = link_inputs(&request, &toolchain, &layout, options.profile, false)?;
    // The exports stay though nothing in the program calls them.
    let mut retained: Vec<String> = request
        .bodies
        .iter()
        .filter(|body| body.abi.is_some() && !body.is_extern_declaration && !body.is_abstract)
        .map(|body| ember_codegen_llvm::function_link_symbol(request.mir.types(), ember_codegen_llvm::Target::host(), body))
        .collect();
    if output.ffi_instrument.is_some() && !cfg!(windows) {
        retained.extend(ember_codegen_c::FFI_INTERPOSED.iter().map(|name| name.to_string()));
    }
    retained.sort();
    retained.dedup();
    let ll_path = sources[0].clone();
    sources.push(runtime_input);
    sources.extend(foreign_objects);
    // `[BLD-4]` — compiled and linked through Ninja, as the C backend's units are: the IR, its shim
    // and the runtime where it is not an object yet, each into its own object under `obj`
    // (`[BLD-13]`: a program database names them), then the objects as they are.
    let (units, objects): (Vec<PathBuf>, Vec<PathBuf>) =
        sources.into_iter().partition(|path| path.extension().is_some_and(|extension| extension == "ll" || extension == "c"));
    ember_build::ninja::build(
        &toolchain,
        &ember_build::ninja::Program {
            units: &units,
            objects: &objects,
            include_dirs: &includes,
            output: exe.clone(),
            profile: options.profile,
            dir: layout.obj.clone(),
            jobs: options.jobs,
            image: None,
        },
        &retained,
        &libraries,
    )
    .map_err(|e| {
        // `[CG-LL-3]` — an instrumented link needs Clang's profile runtime, which some packagings
        // of Clang leave out.
        let error = e.to_string();
        match ember_build::pgo() {
            Some(ember_build::Pgo::Generate(_)) if error.contains("clang_rt.profile") => format!(
                "{error}\n  = help: `--pgo-generate` links Clang's profile runtime, which this Clang lacks: {}",
                if cfg!(windows) { "install LLVM's full package (`winget install --id LLVM.LLVM`)" } else { "on Debian or Ubuntu, `sudo apt install libclang-rt-dev`" }
            ),
            _ => error,
        }
    })?;
    super::mark("LLVM compile and link", "all modules");
    // `[CLI-10]` — Clang's word on the program's loops; `[SIMD-6]` — after Ember's verdicts, its
    // report on the IR.
    let mut engine = request.engine.take();
    if let Some(engine) = engine.as_mut() {
        engine.loops = super::vectorisation_remarks(&toolchain, &ll_path, &includes, &layout, options.profile);
    }
    if options.optimization_report {
        super::print_host_vectorisation_report(&toolchain, &ll_path, &includes, &layout, options.profile);
    }
    super::report(sink, request.map, options);
    if let Some(engine) = &engine {
        engine.print(&toolchain);
    }
    if testing {
        let instrumented = super::instrumented_run(output.ffi_instrument.as_ref(), options, request.bodies, request.modules, request.input, &exe, &toolchain);
        return super::test_runner::run_tests(&exe, request.tests, options, instrumented.as_ref(), request.map);
    }
    if request.command == "run" {
        return super::run_program(&exe, options);
    }
    eprintln!("built {}", exe.display());
    Ok(ExitCode::SUCCESS)
}
