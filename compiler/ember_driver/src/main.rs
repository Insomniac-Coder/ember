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
            let input = args.get(1).ok_or_else(|| format!("`ember {command}` needs a source file"))?;
            if input.starts_with('-') {
                return Err(format!("`ember {command}` needs a source file"));
            }
            let options = parse_options(&args[2..])?;
            compile(Path::new(input), command, &options)
        }
        other => Err(format!("unknown command `{other}`; try `ember --help`")),
    }
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
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // Check.
    let (mut types, common) = TypeTable::new();
    let program = ember_typeck::check(&module, &mut types, &common, &mut sink);
    if options.emit.as_deref() == Some("hir") {
        print!("{}", ember_hir::dump(&program, &types));
        return Ok(finish(&sink, &map, options));
    }
    if sink.has_errors() || command == "check" {
        if !sink.has_errors() && command == "check" {
            report(&sink, &map, options);
            return Ok(ExitCode::SUCCESS);
        }
        return Ok(finish(&sink, &map, options));
    }

    // Lower.
    let bodies = ember_mir::lower(&program, &types);
    if cfg!(debug_assertions) {
        ember_mir::verify::verify_all(&bodies);
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
    let sources = vec![c_path.clone(), runtime.join("src/ember_rt.c")];
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
        return Ok(match status.code() {
            Some(0) => ExitCode::SUCCESS,
            Some(code) => ExitCode::from(code.clamp(0, 255) as u8),
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
        if dir.join("include/ember_rt.h").is_file() {
            return Ok(dir);
        }
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut dir = exe.parent().map(Path::to_path_buf);
    while let Some(candidate) = dir {
        let runtime = candidate.join("runtime/ember_rt");
        if runtime.join("include/ember_rt.h").is_file() {
            return Ok(runtime);
        }
        dir = candidate.parent().map(Path::to_path_buf);
    }
    Err("cannot find the ember_rt sources; set EMBER_RUNTIME_DIR".to_string())
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
