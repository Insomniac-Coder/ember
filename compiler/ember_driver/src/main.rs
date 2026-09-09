//! The `ember` command (Part XIX §1).
//!
//! Phase 0 implements `build`, `run`, `check`, `explain` and the `--emit`
//! stage dumps. The rest of the CLI surface arrives with the phases that give
//! each command something to do.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ember_build::{LinkRequest, Layout, Profile, Toolchain};
use ember_diag::Sink;
use ember_span::SourceMap;
use ember_types::TypeTable;

const USAGE: &str = "\
ember — the Ember compiler

usage:
    ember build <file.em> [options]   compile to an executable
    ember run   <file.em> [options]   compile and run
    ember check <file.em>             type-check without generating code
    ember explain <CODE>              describe a diagnostic code

options:
    --profile debug|release|shipping   default: debug
    --emit tokens|ast|hir|mir|c        print an intermediate form and stop
    --syntax-only                      lex and parse only; report E00xx/E01xx
    --backend c                        the only backend in v1
    --cc msvc|clang|gcc                override C compiler detection
    --out-dir <dir>                    default: target/
    --json                             machine-readable diagnostics
    -D warnings                        treat warnings as errors
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Default)]
struct Options {
    profile: Profile,
    emit: Option<String>,
    cc: Option<String>,
    out_dir: Option<PathBuf>,
    json: bool,
    deny_warnings: bool,
    /// `[CLI-9]` — lex and parse only, reporting `E00xx` and `E01xx`. Names
    /// are not resolved, so an example naming undeclared types still passes.
    /// This is what `[TST-7]`'s gate over the specification's own code blocks
    /// runs.
    syntax_only: bool,
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let Some(command) = args.first() else {
        print!("{USAGE}");
        return Ok(ExitCode::FAILURE);
    };

    match command.as_str() {
        "--help" | "-h" | "help" => {
            print!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        "--version" | "-V" => {
            println!("ember {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        "explain" => {
            let code = args.get(1).ok_or("`ember explain` needs a code, e.g. E3040")?;
            explain(code)
        }
        "build" | "run" | "check" => {
            // The source file may sit before or after the flags. Requiring it
            // first made `ember check --syntax-only f.em` fail with "needs a
            // source file", which is a confusing way to say "wrong order".
            let mut input = None;
            let mut rest = Vec::new();
            let mut skip_value = false;
            for arg in &args[1..] {
                if skip_value {
                    rest.push(arg.clone());
                    skip_value = false;
                    continue;
                }
                if arg.starts_with('-') {
                    skip_value = matches!(
                        arg.as_str(),
                        "--profile" | "--emit" | "--cc" | "--out-dir" | "--backend" | "-D"
                    );
                    rest.push(arg.clone());
                } else if input.is_none() {
                    input = Some(arg.clone());
                } else {
                    rest.push(arg.clone());
                }
            }
            let input = input.ok_or_else(|| format!("`ember {command}` needs a source file"))?;
            let options = parse_options(&rest)?;
            compile(Path::new(&input), command, &options)
        }
        // `[FMT-1]` — the canonical printer. `--check` reports whether the
        // file is already formatted instead of rewriting it.
        "fmt" => {
            let input = args.get(1).ok_or("`ember fmt` needs a source file")?;
            let check_only = args.iter().any(|a| a == "--check");
            let write = args.iter().any(|a| a == "--write");
            format_file(Path::new(input), check_only, write)
        }
        other => Err(format!("unknown command `{other}`; try `ember --help`")),
    }
}

fn format_file(input: &Path, check_only: bool, write: bool) -> Result<ExitCode, String> {
    let mut map = SourceMap::new();
    let file = map.load(input).map_err(|e| e.to_string())?;
    let source = map.file(file).text.clone();

    let mut sink = Sink::new();
    let lexed = ember_lexer::lex(file, &source, &mut sink);
    let module = ember_parser::parse(file, &source, lexed.tokens, &mut sink);
    if sink.has_errors() {
        report(&sink, &map, &Options::default());
        return Ok(ExitCode::FAILURE);
    }

    let formatted = ember_fmt::format(&module, &source, &lexed.comments);
    if check_only {
        if formatted == source {
            return Ok(ExitCode::SUCCESS);
        }
        eprintln!("{} is not formatted", input.display());
        return Ok(ExitCode::FAILURE);
    }
    if write {
        std::fs::write(input, formatted).map_err(|e| e.to_string())?;
        return Ok(ExitCode::SUCCESS);
    }
    print!("{formatted}");
    Ok(ExitCode::SUCCESS)
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        let value = |index: &mut usize, name: &str| -> Result<String, String> {
            *index += 1;
            args.get(*index).cloned().ok_or_else(|| format!("`{name}` needs a value"))
        };
        match arg {
            "--profile" => options.profile = Profile::from_name(&value(&mut index, arg)?)
                .ok_or("profile must be debug, release or shipping")?,
            "--emit" => options.emit = Some(value(&mut index, arg)?),
            "--cc" => options.cc = Some(value(&mut index, arg)?),
            "--out-dir" => options.out_dir = Some(PathBuf::from(value(&mut index, arg)?)),
            "--backend" => {
                let backend = value(&mut index, arg)?;
                if backend != "c" {
                    return Err("the only backend in v1 is `c`; LLVM is v2".to_string());
                }
            }
            "--syntax-only" => options.syntax_only = true,
            "--json" => options.json = true,
            "-Dwarnings" | "-D" => {
                if arg == "-D" {
                    let what = value(&mut index, arg)?;
                    if what != "warnings" {
                        return Err(format!("unknown `-D` argument `{what}`"));
                    }
                }
                options.deny_warnings = true;
            }
            other => return Err(format!("unknown option `{other}`")),
        }
        index += 1;
    }
    Ok(options)
}

/// `[DIA-7]` — `target/<profile>/unclassified-borrow-errors.log`.
///
/// Written only when there is something to write, so its presence is the
/// signal: an empty run leaves no file, and CI checks for the file rather than
/// parsing output.
fn write_unclassified_log(sink: &ember_diag::Sink, options: &Options) {
    let entries = sink.unclassified();
    if entries.is_empty() {
        return;
    }
    let dir = options
        .out_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("target").join(options.profile.name()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("unclassified-borrow-errors.log");
    let body = entries.join("
") + "
";
    let _ = std::fs::write(&path, body);
    eprintln!(
        "note: {} unclassified borrow error(s) recorded in {} (DIA-7)",
        entries.len(),
        path.display()
    );
}

fn explain(code: &str) -> Result<ExitCode, String> {
    let Some(found) = ember_diag::codes::lookup(code) else {
        return Err(format!("`{code}` is not a diagnostic code"));
    };
    println!("{found}: {}", found.title);
    println!();
    println!("Specified by {} ({:?}).", found.rule, found.subsystem);
    // `[DIA-6]` — docs/errors/EXXXX.md exists for every code. The pages are
    // written as each code gains its first test; until then, say so plainly
    // rather than printing an empty page.
    let page = Path::new("docs/errors").join(format!("{found}.md"));
    match std::fs::read_to_string(&page) {
        Ok(text) => {
            println!();
            print!("{text}");
        }
        Err(_) => {
            println!();
            println!("No extended description has been written for this code yet.");
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// `[MOD-1]` — every module reachable from the root, in load order with the
/// root first.
///
/// `[MOD-4]` allows import cycles inside a package, so a module already loaded
/// is skipped rather than reported.
fn load_modules(
    root: ember_ast::Module,
    root_dir: &Path,
    map: &mut SourceMap,
    sink: &mut Sink,
) -> Vec<ember_typeck::LoadedModule> {
    let mut loaded: Vec<ember_typeck::LoadedModule> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut queue: Vec<(Vec<String>, ember_ast::Module)> = vec![(Vec::new(), root)];

    while let Some((path, module)) = queue.pop() {
        // Every import this module names, before it is moved into the list.
        let mut wanted: Vec<(Vec<String>, ember_span::Span)> = Vec::new();
        for import in &module.imports {
            let segments = match &import.kind {
                ember_ast::ImportKind::Module { path, .. } => path,
                ember_ast::ImportKind::Items { path, .. } => path,
                // `import c "header.h"` is Phase 5's.
                ember_ast::ImportKind::Foreign { .. } => continue,
            };
            let names: Vec<String> = segments.iter().map(|s| s.name.to_string()).collect();
            wanted.push((names, import.span));
        }

        loaded.push(ember_typeck::LoadedModule { path: path.clone(), module });

        for (names, span) in wanted {
            let key = names.join(".");
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key.clone());
            let Some(file_path) = resolve_module(root_dir, &names) else {
                // `[MOD-5]`'s prelude names are compiler-known until the
                // library can supply each one, so an import of a `std` module
                // that has no file yet is not an error — it is a module this
                // phase has not written.
                if names.first().is_some_and(|f| f == ember_branding::STD_PACKAGE) {
                    continue;
                }
                sink.emit(ember_diag::Diagnostic::error(
                    ember_diag::codes::E1010,
                    span,
                    format!("cannot find module `{key}`"),
                ));
                continue;
            };
            let file = match map.load(&file_path) {
                Ok(file) => file,
                Err(error) => {
                    sink.emit(ember_diag::Diagnostic::error(
                        ember_diag::codes::E1010,
                        span,
                        format!("cannot read `{}`: {error}", file_path.display()),
                    ));
                    continue;
                }
            };
            let text = map.file(file).text.clone();
            let lexed = ember_lexer::lex(file, &text, sink);
            let parsed = ember_parser::parse(file, &text, lexed.tokens, sink);
            queue.push((names, parsed));
        }
    }
    loaded
}

/// `[MOD-1]`, `[MOD-3]` — where a module path's file is.
///
/// A path beginning `std` names the standard library package, whose sources
/// are found by `ember_branding::std_root`; anything else is a module of the
/// package being built and is found relative to its root. The two are
/// separate roots because `[MOD-1]` makes a module path "package name + path
/// from `src/`", so `std.span` is `span.em` under `std`'s own `src/` and not
/// `std/span.em` under this package's.
fn resolve_module(root_dir: &Path, names: &[String]) -> Option<std::path::PathBuf> {
    if names.first().is_some_and(|f| f == ember_branding::STD_PACKAGE) {
        let std_root = ember_branding::std_root()?;
        // `import std` alone names the package root module.
        if names.len() == 1 {
            let root = std_root.join(ember_branding::source_file("lib"));
            return root.is_file().then_some(root);
        }
        return module_file(&std_root, &names[1..]);
    }
    module_file(root_dir, names)
}

/// `[MOD-1]` — a module path maps to `a/b/c.em`, or to `a/b/c/mod.em` when
/// the directory has submodules of its own.
fn module_file(root: &Path, segments: &[String]) -> Option<std::path::PathBuf> {
    let mut direct = root.to_path_buf();
    for segment in segments {
        direct.push(segment);
    }
    let flat = direct.with_extension(ember_branding::SOURCE_EXT);
    if flat.is_file() {
        return Some(flat);
    }
    let nested = direct.join(ember_branding::source_file("mod"));
    if nested.is_file() {
        return Some(nested);
    }
    None
}

fn compile(input: &Path, command: &str, options: &Options) -> Result<ExitCode, String> {
    let mut map = SourceMap::new();
    let file = map.load(input).map_err(|e| e.to_string())?;
    let source = map.file(file).text.clone();

    let mut sink = Sink::new();
    sink.deny_warnings = options.deny_warnings;

    // Lex.
    let lexed = ember_lexer::lex(file, &source, &mut sink);
    if options.emit.as_deref() == Some("tokens") {
        for token in &lexed.tokens {
            println!("{:?} {:?}", token.kind, token.span);
        }
        return Ok(finish(&sink, &map, options));
    }

    // Parse.
    let module = ember_parser::parse(file, &source, lexed.tokens, &mut sink);
    if options.emit.as_deref() == Some("ast") {
        print!("{}", ember_ast::dump(&module));
        return Ok(finish(&sink, &map, options));
    }
    // `[CLI-9]` — stop here. Everything after this point resolves names.
    if options.syntax_only {
        return Ok(finish(&sink, &map, options));
    }
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // `[MOD-1]` — follow the imports and load every module they reach. The
    // root module is the file named on the command line; its directory is the
    // package root until `ember.toml` is read.
    let root_dir = input.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    let modules = load_modules(module, &root_dir, &mut map, &mut sink);
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // Check.
    let (mut types, common) = TypeTable::new();
    // [TYP-8] -- the profile chooses the default overflow policy.
    let program =
        ember_typeck::check(&modules, &mut types, &common, &mut sink, overflow_policy(options.profile));
    if options.emit.as_deref() == Some("hir") {
        print!("{}", ember_hir::dump(&program, &types));
        return Ok(finish(&sink, &map, options));
    }
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // Lower. `ember check` runs the MIR analyses too — Part XIX §1 defines it
    // as "type-check + borrow-check without codegen", so it cannot stop here.
    let mut bodies = ember_mir::lower(&program, &types, &common);
    if cfg!(debug_assertions) {
        ember_mir::verify::verify_all(&bodies);
    }
    // Definite initialisation (Part XVIII §4.6) runs on MIR, before any
    // optimisation could remove the read it is looking for.
    ember_analysis::check_definite_init_all(&bodies, &mut sink);
    // `[OWN-3]`, Part XVIII §4.9 — moves are tracked, a use after a move is
    // `E3040`, and the drops lowering inserted are removed where the value was
    // moved away or made conditional on a drop flag where it may have been.
    ember_analysis::elaborate_drops_all(&mut bodies, &types, &mut sink);
    // `[LNT-3]` — `L1001`/`L1002` are emitted by `build` and `check`, not only
    // by `ember lint`. A lint that fires on a separate command does not close
    // the footgun `[GRM-4]` opens.
    // Part XVIII §4.7 — the NLL borrow checker runs on MIR after drop
    // elaboration, so the drops it sees are the ones that will exist.
    ember_analysis::check_borrows_all(&bodies, &types, &mut sink);
    ember_analysis::check_unused_all(&bodies, &mut sink);
    // `[DIA-7]` — a borrow error the classifier could not place is recorded
    // rather than left to be noticed. CI fails when the conformance suite
    // produces any, which is what stops an unexplained rejection shipping.
    write_unclassified_log(&sink, options);
    if cfg!(debug_assertions) {
        ember_mir::verify::verify_all(&bodies);
        // D-022's backstop. Every view in the finished MIR must have arrived
        // from a borrow the analyses can see; a view-producing path that
        // forgot to take one leaves a shape this recognises. Checked here
        // rather than at lowering because it must hold of the MIR the borrow
        // checker actually ran on.
        ember_mir::verify::verify_views_all(&bodies, &types);
    }
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }
    if command == "check" {
        report(&sink, &map, options);
        return Ok(ExitCode::SUCCESS);
    }

    if options.emit.as_deref() == Some("mir") {
        print!("{}", ember_mir::dump(&bodies, &types));
        return Ok(finish(&sink, &map, options));
    }

    // Emit C.
    let module_name = input.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let emitted = ember_codegen_c::emit(&bodies, &types, &map, &module_name, program.main.is_some());
    if options.emit.as_deref() == Some("c") {
        print!("{}", emitted.c_source);
        return Ok(finish(&sink, &map, options));
    }
    if let Some(other) = &options.emit {
        return Err(format!("unknown `--emit` stage `{other}`"));
    }

    if program.main.is_none() {
        return Err("this file declares no `main`, so there is nothing to run".to_string());
    }

    // Write, compile and link.
    let target_dir = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target"));
    let layout = Layout::new(&target_dir, options.profile).map_err(|e| e.to_string())?;
    let c_path = layout.c.join(format!("{module_name}.c"));
    std::fs::write(&c_path, &emitted.c_source).map_err(|e| e.to_string())?;

    let runtime = runtime_dir()?;
    let exe_name = if cfg!(windows) { format!("{module_name}.exe") } else { module_name.clone() };
    let exe = layout.bin.join(exe_name);

    let toolchain = Toolchain::detect(options.cc.as_deref()).map_err(|e| e.to_string())?;
    let runtime_source = format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX);
    let sources = vec![c_path.clone(), runtime.join(runtime_source)];
    let includes = vec![runtime.join("include")];
    ember_build::compile_and_link(
        &toolchain,
        &LinkRequest {
            sources: &sources,
            include_dirs: &includes,
            output: exe.clone(),
            profile: options.profile,
            obj_dir: layout.obj.clone(),
        },
    )
    .map_err(|e| e.to_string())?;

    report(&sink, &map, options);

    if command == "run" {
        let status = std::process::Command::new(&exe).status().map_err(|e| e.to_string())?;
        // A panic calls abort(), and Windows reports that as a status well
        // outside 0..=255 (0xC0000409 arrives as a large negative i32).
        // Clamping it into a u8 turned a crash into a clean exit, so anything
        // that is not representable becomes a plain failure.
        return Ok(match status.code() {
            Some(0) => ExitCode::SUCCESS,
            Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
            None => ExitCode::FAILURE,
        });
    }

    eprintln!("built {}", exe.display());
    Ok(ExitCode::SUCCESS)
}

/// Where `ember_rt`'s sources live. Found relative to the compiler executable
/// during development; an installed toolchain will carry them alongside the
/// binary.
fn runtime_dir() -> Result<PathBuf, String> {
    let from_env = std::env::var_os("EMBER_RUNTIME_DIR").map(PathBuf::from);
    if let Some(dir) = from_env {
        if dir.join(format!("include/{}", ember_branding::runtime_header())).is_file() {
            return Ok(dir);
        }
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut dir = exe.parent().map(Path::to_path_buf);
    while let Some(candidate) = dir {
        let runtime = candidate.join(format!("runtime/{}_rt", ember_branding::SYMBOL_PREFIX));
        if runtime.join(format!("include/{}", ember_branding::runtime_header())).is_file() {
            return Ok(runtime);
        }
        dir = candidate.parent().map(Path::to_path_buf);
    }
    Err(format!(
        "cannot find the {}_rt sources; set EMBER_RUNTIME_DIR",
        ember_branding::SYMBOL_PREFIX
    ))
}

/// `[TYP-8]`, and Part XIX §2's profile table: `debug` panics on overflow,
/// `release` and `shipping` wrap. `[PRF-1]` allows exactly this one difference
/// in semantics between profiles.
fn overflow_policy(profile: Profile) -> ember_types::OverflowPolicy {
    match profile {
        Profile::Debug => ember_types::OverflowPolicy::Panic,
        Profile::Release | Profile::Shipping => ember_types::OverflowPolicy::Wrap,
    }
}

fn report(sink: &Sink, map: &SourceMap, options: &Options) {
    if sink.diagnostics().is_empty() {
        return;
    }
    if options.json {
        print!("{}", sink.render_json(map));
    } else {
        eprint!("{}", sink.render(map));
    }
}

fn finish(sink: &Sink, map: &SourceMap, options: &Options) -> ExitCode {
    report(sink, map, options);
    if sink.has_errors() {
        let errors = sink.error_count();
        if errors > 0 {
            eprintln!("error: could not compile due to {errors} error(s)");
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
