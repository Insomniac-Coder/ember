//! The `ember` command (Part XIX §1).
//!
//! Phase 0 implements `build`, `run`, `check`, `explain` and the `--emit`
//! stage dumps. The rest of the CLI surface arrives with the phases that give
//! each command something to do.

mod bench;
mod cimport;
mod comptime;
mod foreign;
mod packages;
mod reload;
mod test_runner;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ember_build::interface::{
    CallableGenericCallableBound, CallableGenericParameter, CallableInterfaceContract,
    CallableInterfaceRecord, CallableParameter, CallableParameterMode, CallableSignature,
    InterfaceHash, ModuleInterfaceArtifact, ModuleInterfaceInput, PreparedInterfaceCache,
    build_artifacts, cache_directory, compiler_identity, prepare_interface_cache,
    round_trip_artifacts,
};
use ember_build::{Layout, LinkRequest, Profile, Toolchain};
use ember_diag::{Diagnostic, Sink, codes};
use ember_span::{LoadError, SourceMap, Span};
use ember_types::TypeTable;

const USAGE: &str = "\
ember — the Ember compiler

usage (`[CLI-15]`: every command and flag of the specification's XVII.1; \"not built yet\" marks those
this compiler does not have yet):
    ember new <name> [--lib | --bin | --cdylib]    create a package
    ember run [file.em] [args…] [--hot | --interp] build and run; a single file needs no manifest;
                                                   --hot reloads edited sources at std.hot.checkpoint()
                                                   (--interp is not built yet)
    ember build [file.em|package-dir] [--profile <name>] [--target <triple>] [--backend c|llvm]
                [--emit tokens|ast|hir|mir|c|obj|header] [--out-dir <dir>]
                [--emit-optimization-report] [--report=engine|instantiations] [--timings[=json]]
                [--build-id] [--reload [--explain]] [-D warnings]
                                                   compile a bin, staticlib or cdylib package; --reload
                                                   builds the next image of a running program and
                                                   --explain predicts that reload; --build-id prints
                                                   the hash of the build's inputs; --report=engine
                                                   what the build did to the program (--target and
                                                   --backend llvm are not built yet)
    ember check [file.em] [--syntax-only] [--timings]
                                                   type, borrow and effect checks without code generation
    ember test [filter] [--doc] [--no-leak-check] [--instrument-ffi]
                                                   run the package's @test functions, each in a process
                                                   of its own; --doc runs the documentation examples;
                                                   --instrument-ffi grades foreign effect facts by
                                                   what the tests observe
    ember bench [filter] --compare [--core N] [--report]
                                                   time tests/perf/'s programs against their C twins on
                                                   one core, each at its fastest code layout; fails
                                                   past a threshold ([TST-28]), or only reports
                                                   (--report, off the reference machine)
    ember fmt <file.em> [--check | --write | --migrate] [--json]
                                                   print the file in the canonical layout
                                                   (--migrate: 0.9.8 source rewritten into 0.9.9)
    ember lint [file.em] [--json]                  check, then report [LNT-6]'s lints: unneeded clones,
                                                   large copies, guards held too long, allocation and
                                                   dispatch in hot loops, copies at the FFI boundary
    ember doc                                      (not built yet)
    ember update [package]                         re-resolve dependencies and rewrite ember.lock
                                                   (not built yet)
    ember explain <E-code | rule-id> [--json]      the error page or rule text
    ember explain --borrow <file>:<line>           why each borrow live at that line is still live
    ember explain --cycle <path> <Class[.field]>   one cycle-capable class or field
    ember inspect <item> [--safety [--elided-only] | --alloc | --cost | --deterministic <fn> | --cycle
                          | --expand | --counts | --generics] [--function <name>]
                                                   reports on a program (--expand is not built yet)
    ember audit <file> [--json]                    a one-page summary of the safety surface: unsafe code,
                                                   effects, the foreign boundary, the trusted base
    ember why --alloc|--block|--io|--lock|--sync|--unsafe|--ffi <fn> <file>
                                                   the shortest call chain to that effect
    ember calls --foreign <item>                   every foreign function the item can reach
    ember bind <header> [-I <dir>] [-D <name[=value]>] (--init [--merge] | --report [--overlay <file>]
                         [--baseline <file> | --write-baseline <file>] | --check <overlay>)
                                                   a starter overlay, what is skipped or still unsafe,
                                                   an overlay checked against the header
                                                   (--explain and --emit-embind are not built yet)
    ember shader-bind <reflection.json>            Annex D (not built yet)
    ember tcb [--module <m>] [--json] <file.em>    the trusted-base report (Annex C)
    ember toolchain list|install|default           manage installed C toolchains (not built yet)
    ember clean                                    (not built yet)
    ember --help | --version [--matrix]            (--matrix is not built yet)

options:
    --profile debug|development|release   default: debug
    --emit tokens|ast|hir|mir|c|obj|header
                                      emit a stage and stop; obj and header write files
    --emit-header                     write <package>.h alongside a build or C emit
    --emit-optimization-report        list the calls made direct on the whole program's word
    --timings[=json]                  each stage's time, module by module where it runs so
    --syntax-only                     lex and parse only; report E00xx/E01xx
    --backend c                       the only backend in v1
    --cc msvc|clang|gcc|clang-cl      override C compiler detection
    --out-dir <dir>                   default: target/; libraries and headers in <profile>/lib/
    --leak-check                      `run` and `test`: report leaked ownership cycles at exit
                                      (the default for both in the debug profile)
    --no-leak-check                   `run` and `test`: no leak report in the debug profile
    --json                            machine-readable output and diagnostics (every command)
    -D warnings                       treat warnings as errors

inspect options:
    --function <name>                 restrict the report to one function
    --elided-only                     report only statically elided checks
";

/// The compiler's stack: the same on every host (D-330). Checking and
/// lowering recurse once for each level of an expression, and a debug build
/// spends about 65 KB of stack on a level of a binary operator, so the 1 MB
/// Windows gives a program's main thread held a twelve-deep polynomial and no
/// more, where Linux's 8 MB held a hundred levels. The size is reserved, not
/// committed: a host spends only what a program's nesting uses.
const COMPILER_STACK: usize = 256 << 20;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Named as the main thread is, which an internal error's message names.
    let compiler = std::thread::Builder::new()
        .name("main".to_string())
        .stack_size(COMPILER_STACK)
        .spawn(move || compile_command(&args));
    match compiler {
        // A panic, an internal error, was reported on the thread; it ends
        // the process as it would have on the main thread.
        Ok(thread) => thread
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic)),
        Err(error) => {
            eprintln!("error: cannot start the compiler's thread: {error}");
            ExitCode::FAILURE
        }
    }
}

fn compile_command(args: &[String]) -> ExitCode {
    match run(args) {
        Ok(code) => code,
        // A message that is already a coded diagnostic (`error[E9002]: …`, `[TOOL-3]`) prints as it is.
        Err(message) if message.starts_with("error[") => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Keep implicit standard-library loading from changing unrelated generated
/// programs. `[MOD-5]` makes source-backed prelude interfaces available even
/// without an import, but availability is not reachability: an unused
/// `DefaultHasher` body must not inject bounds/overflow panic paths into every
/// executable. All user/package bodies remain present for the current
/// compilation-unit model; only unreferenced `std` bodies are removed.
///
/// Direct calls and function constants form the current executable call graph.
/// Standard `drop` bodies remain roots because drop glue names them from type
/// information rather than through an explicit MIR call terminator.
///
/// A body is `std`'s when its symbol is in `std`'s namespace or it was written
/// in one of `std`'s files: `std.core`'s `extend i8 implements
/// FloorDiv[NonZero[i8]]` makes `i8_floordiv`, named after `i8`, and before
/// D-339 each such body was emitted into every program.
fn retain_referenced_standard_bodies(
    bodies: &mut Vec<ember_mir::Body>,
    standard_files: &std::collections::BTreeSet<ember_span::FileId>,
    types: &TypeTable,
) {
    use std::collections::{BTreeMap, BTreeSet};

    let std_prefix = ember_branding::mangled(&format!("{}.", ember_branding::STD_PACKAGE));
    let standard =
        |body: &ember_mir::Body| body.symbol.starts_with(&std_prefix) || standard_files.contains(&body.span.file);
    let by_symbol: BTreeMap<String, usize> = bodies
        .iter()
        .enumerate()
        .map(|(index, body)| (body.symbol.clone(), index))
        .collect();
    let mut keep = BTreeSet::new();
    let mut pending = Vec::new();
    // A virtual call reaches every body in its slot: a standard class's override is called
    // through its base's slot, never by name.
    let mut by_slot: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for body in bodies.iter().filter(|body| standard(body)) {
        if let Some(slot) = body.class_virtual_slot {
            by_slot.entry(slot).or_default().push(body.symbol.clone());
        }
    }

    for body in bodies.iter() {
        let drop_glue = body.name == "drop" || body.name.ends_with(".drop");
        if (!standard(body) || drop_glue) && keep.insert(body.symbol.clone()) {
            pending.push(body.symbol.clone());
        }
    }

    while let Some(symbol) = pending.pop() {
        let Some(&index) = by_symbol.get(&symbol) else {
            continue;
        };
        let mut referenced = BTreeSet::new();
        collect_body_function_symbols(&bodies[index], &mut referenced);
        // ADR-186 — formatting a value runs the written `Debug` of each part
        // of it that has one, which the backend calls.
        for block in &bodies[index].blocks {
            if let ember_mir::Terminator::Call {
                func:
                    ember_mir::FuncRef::Builtin {
                        which:
                            ember_mir::Builtin::Format
                            | ember_mir::Builtin::FormatWith(_)
                            | ember_mir::Builtin::Print
                            | ember_mir::Builtin::EPrint,
                        arg_ty,
                    },
                ..
            } = &block.terminator
            {
                referenced.extend(types.debug_writers_reached(*arg_ty).into_iter().map(str::to_string));
            }
            if let ember_mir::Terminator::Call { func: ember_mir::FuncRef::Virtual { slot, .. }, .. } = &block.terminator {
                referenced.extend(by_slot.get(slot).into_iter().flatten().cloned());
            }
        }
        for target in referenced {
            if by_symbol.contains_key(&target) && keep.insert(target.clone()) {
                pending.push(target);
            }
        }
    }

    bodies.retain(|body| !standard(body) || keep.contains(&body.symbol));
}

/// `[TYP-9c]` — a C compiler that ignored or refused the flag relaxing a unit's floating point
/// cannot honour `@fastmath` or `@fp(contract)`: `E9041`, naming it and the attribute, at the first
/// function written with it. An attribute is never ignored (`[PHIL-12]`). Whether `error` was that.
fn report_float_mode_refused(sink: &mut Sink, bodies: &[ember_mir::Body], error: &ember_build::BuildError) -> bool {
    let ember_build::BuildError::FloatModeRefused { toolchain, flag, fast } = error else { return false };
    let (attribute, mode) =
        if *fast { ("@fastmath", ember_types::FpMode::Fast) } else { ("@fp(contract)", ember_types::FpMode::Contract) };
    let span = bodies.iter().find(|body| body.fp == mode && !body.span.is_dummy()).map_or(Span::DUMMY, |body| body.span);
    sink.emit(
        Diagnostic::error(codes::E9041, span, format!("`{toolchain}` cannot compile a function with `{attribute}` as written: it does not take `{flag}`"))
            .help("build with a C compiler that takes it (MSVC from Visual Studio 2022 17.0, clang or gcc), or remove the attribute")
            .note("an attribute is never ignored, and its floating point is built per function [TYP-9c]"),
    );
    true
}

fn verify_callable_regions_or_panic(bodies: &[ember_mir::Body], types: &TypeTable) {
    let violations = ember_analysis::verify_callable_regions_all(bodies, types);
    if !violations.is_empty() {
        panic!(
            "MIR callable-region verification failed:\n{}",
            violations
                .iter()
                .map(|v| format!("  {}: {}", v.body, v.message))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

fn collect_body_function_symbols(
    body: &ember_mir::Body,
    out: &mut std::collections::BTreeSet<String>,
) {
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                ember_mir::StmtKind::Assign { rvalue, .. } => {
                    collect_rvalue_function_symbols(rvalue, out);
                }
                ember_mir::StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    collect_operand_function_symbols(lhs, out);
                    collect_operand_function_symbols(rhs, out);
                }
                ember_mir::StmtKind::StorageLive(_)
                | ember_mir::StmtKind::StorageDead(_)
                | ember_mir::StmtKind::Drop { .. }
                | ember_mir::StmtKind::BeginAccess { .. }
                | ember_mir::StmtKind::BeginAccessTransfer { .. }
                | ember_mir::StmtKind::EndAccess { .. }
                | ember_mir::StmtKind::EndAccessTransfer { .. }
                | ember_mir::StmtKind::LendAccess { .. }
                | ember_mir::StmtKind::Nop => {}
            }
        }
        match &block.terminator {
            ember_mir::Terminator::SwitchInt { discr, .. } => {
                collect_operand_function_symbols(discr, out);
            }
            ember_mir::Terminator::Call { func, args, .. } => {
                match func {
                    ember_mir::FuncRef::Direct { symbol, .. } => {
                        out.insert(symbol.clone());
                    }
                    ember_mir::FuncRef::Indirect { operand, .. } => {
                        collect_operand_function_symbols(operand, out);
                    }
                    ember_mir::FuncRef::Builtin { .. } => {}
                    ember_mir::FuncRef::DynBoxNew { implementations, .. } => {
                        out.extend(
                            implementations
                                .iter()
                                .flatten()
                                .map(|implementation| implementation.symbol.clone()),
                        );
                    }
                    ember_mir::FuncRef::Virtual { .. } => {}
                    ember_mir::FuncRef::Interface { .. } => {}
                }
                for arg in args {
                    collect_operand_function_symbols(arg, out);
                }
            }
            ember_mir::Terminator::Assert { cond, msg, .. } => {
                collect_operand_function_symbols(cond, out);
                match msg {
                    ember_mir::AssertKind::Bounds { len, index } => {
                        collect_operand_function_symbols(len, out);
                        collect_operand_function_symbols(index, out);
                    }
                    ember_mir::AssertKind::RefCellBorrow { file, line } => {
                        collect_operand_function_symbols(file, out);
                        collect_operand_function_symbols(line, out);
                    }
                    ember_mir::AssertKind::Overflow(_)
                    | ember_mir::AssertKind::DivisionByZero
                    | ember_mir::AssertKind::SignedDivisionOverflow
                    | ember_mir::AssertKind::ShiftTooLarge
                    | ember_mir::AssertKind::Downcast => {}
                    ember_mir::AssertKind::Panic { message } => {
                        collect_operand_function_symbols(message, out);
                    }
                }
            }
            ember_mir::Terminator::Goto(_)
            | ember_mir::Terminator::Return
            | ember_mir::Terminator::Unreachable => {}
        }
    }
}

fn collect_rvalue_function_symbols(
    rvalue: &ember_mir::Rvalue,
    out: &mut std::collections::BTreeSet<String>,
) {
    match rvalue {
        // D-602 — a `ref dyn I` (or class-interface) table names the bodies
        // that fill it, as a `Box[dyn I]`'s does: one only a table names
        // was dropped, and the table's C named a function never emitted.
        ember_mir::Rvalue::Cast {
            kind:
                ember_mir::CastKind::InterfaceUpcast { implementations, .. }
                | ember_mir::CastKind::ClassInterfaceUpcast { implementations, .. },
            operand,
            ..
        } => {
            out.extend(implementations.iter().flatten().map(|implementation| implementation.symbol.clone()));
            collect_operand_function_symbols(operand, out);
        }
        ember_mir::Rvalue::Use(operand)
        | ember_mir::Rvalue::UnaryOp { operand, .. }
        | ember_mir::Rvalue::Cast { operand, .. } => collect_operand_function_symbols(operand, out),
        ember_mir::Rvalue::BinaryOp { lhs, rhs, .. } => {
            collect_operand_function_symbols(lhs, out);
            collect_operand_function_symbols(rhs, out);
        }
        ember_mir::Rvalue::Aggregate { operands, .. } => {
            for operand in operands {
                collect_operand_function_symbols(operand, out);
            }
        }
        ember_mir::Rvalue::Repeat { value, .. } => collect_operand_function_symbols(value, out),
        ember_mir::Rvalue::Discriminant(_) | ember_mir::Rvalue::Ref { .. } => {}
    }
}

fn collect_operand_function_symbols(
    operand: &ember_mir::Operand,
    out: &mut std::collections::BTreeSet<String>,
) {
    if let ember_mir::Operand::Const(ember_mir::Const::Fn(symbol)) = operand {
        out.insert(symbol.clone());
    }
}

#[derive(Default)]
struct Options {
    profile: Profile,
    emit: Option<String>,
    emit_header: bool,
    cc: Option<String>,
    out_dir: Option<PathBuf>,
    json: bool,
    deny_warnings: bool,
    /// `[WK-8]` — emit the runtime cycle inspector into the generated entry
    /// point without changing the Ember program's command-line arguments.
    /// `[WK-15]`: on for `ember run` in the debug profile unless
    /// `--no-leak-check`.
    leak_check: bool,
    no_leak_check: bool,
    /// `[DSP-5]` — `--emit-optimization-report`: print the optimisations the
    /// whole program allowed, each at its source location.
    optimization_report: bool,
    /// `[BLD-9]` — `--timings` (`Some(false)`) or `--timings=json` (`Some(true)`).
    timings: Option<bool>,
    /// `[CLI-9]` — lex and parse only, reporting `E00xx` and `E01xx`. Names
    /// are not resolved, so an example naming undeclared types still passes.
    /// This is what `[TST-7]`'s gate over the specification's own code blocks
    /// runs.
    syntax_only: bool,
    /// XVII.1's `ember run [file.em] [args…]`: the program's own
    /// arguments, `main`'s `args` after its name (`[FN-8]`).
    program_args: Vec<String>,
    /// `[PHIL-2]` — `ember inspect --alloc`: check the program, then list
    /// its allocation sites instead of generating code.
    alloc_report: Option<AllocReport>,
    /// `[TST-3]` — `ember test [filter]`: run only the tests whose name
    /// contains it.
    test_filter: Option<String>,
    /// `[UNS-8]` — `ember tcb`: check the program, then list its `unsafe` instead of generating
    /// code.
    tcb_report: Option<TcbReport>,
    /// `[BUD-2]`'s B2, Annex B — `ember build --reload`: build the next image of a running reloadable
    /// program and announce it to the program's polls.
    reload: bool,
    /// `[HR-18b]` — `ember build --reload --explain`: predict the reload, apply nothing.
    explain: bool,
    /// `[HR-33]` — `ember run --hot`: run, watch the sources, rebuild and reload.
    hot: bool,
    /// `[MAN-2]` — `--locked`: a build that would change `ember.lock` fails.
    locked: bool,
    /// `[CLI-13]` — `ember calls --foreign <item>`: check the program, then list the foreign
    /// functions the item can reach.
    foreign_calls: Option<ForeignCalls>,
    /// `[CLI-12]`, `[DET-9]` — `ember why` and `ember inspect --deterministic`.
    effect_report: Option<EffectReport>,
    /// `[MONO-2]` — `--report=instantiations`.
    instantiation_report: bool,
    /// `[DIA-8]` — `ember explain --borrow <file>:<line>`: the loans live at that line.
    borrow_report: Option<u32>,
    /// `[DET-7]` — `--build-id`: print the hash of the build's inputs.
    build_id: bool,
    /// `[CLI-10]` — `--report=engine`: what the build did to the program, changing nothing.
    engine_report: bool,
    /// `[FFI-37]` — `ember test --instrument-ffi`: the directory its evidence record goes to
    /// (`[TCB-1]`'s `[ffi] evidence`).
    ffi_evidence: Option<PathBuf>,
    /// `[TCB-1]` — `ember build --require-evidence`, `ember tcb --require`: a claimed grade whose
    /// evidence is missing or stale is `E5050`, not `W5050`.
    require_evidence: bool,
    /// `[LNT-6]` — `ember lint`: check, then report the lints `check` leaves out.
    lint: bool,
    /// `[MONO-7]` — `ember inspect --generics`: check, then say of each generic whether its
    /// instances were specialised or share one body, and why.
    generics_report: bool,
}

/// What `ember calls --foreign` prints: JSON or text, for one item.
struct ForeignCalls {
    json: bool,
    item: String,
}

/// `[CLI-12]` — the shortest chain from `item` to an operation with `effect`; `[DET-9]`
/// (`deterministic`) — every `Nondet` source `item` reaches.
struct EffectReport {
    effect: ember_hir::Effects,
    item: String,
    json: bool,
    deterministic: bool,
}

/// What `ember tcb` prints: JSON or text, one module or all.
#[derive(Clone, Default)]
struct TcbReport {
    json: bool,
    module: Option<String>,
    /// `[CLI-11]` — `ember audit`: the summary first, then the trusted base.
    audit: bool,
}

/// What `ember inspect --alloc` prints: JSON or text, one function or all.
struct AllocReport {
    json: bool,
    function: Option<String>,
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let Some(command) = args.first() else {
        print!("{USAGE}");
        return Ok(ExitCode::FAILURE);
    };

    // `[CLI-1]` — every command accepts `--json`.
    let json = args.iter().skip(1).any(|argument| argument == "--json");
    match command.as_str() {
        "--help" | "-h" | "help" => {
            if json {
                println!("{}", serde_json::json!({ "schema": 1, "usage": USAGE }));
            } else {
                print!("{USAGE}");
            }
            Ok(ExitCode::SUCCESS)
        }
        "--version" | "-V" => {
            // `[CONF-1]` — the profile this compiler claims. Raise it only when every rule of the next
            // profile is done: `tools/phase_progress.py --check` compares it with the evidence
            // (`docs/conformance.json`) and fails CI on any difference, either way.
            const PROFILE: &str = "none";
            if args.iter().any(|argument| argument == "--matrix") {
                return Err("error[E0900]: `ember --version --matrix` is not implemented yet".to_string());
            }
            // `[TOOL-4]` — the compiler, the language, the backend, and the C compiler and linker
            // this build resolves (`--cc`'s variable steers it, as for a build), so a bug report
            // carries its environment.
            let requested = std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty());
            let toolchain = Toolchain::detect(requested.as_deref());
            let (compiler, linker) = match &toolchain {
                Ok(toolchain) => (format!("{} ({})", toolchain.name(), toolchain.compiler_path().display()), toolchain.linker()),
                Err(error) => (format!("none found: {error}"), "none".to_string()),
            };
            if json {
                println!("{}", serde_json::json!({
                    "schema": 1, "version": env!("CARGO_PKG_VERSION"), "language": ember_parser::LANGUAGE_VERSION,
                    "backend": "c", "c_compiler": compiler, "linker": linker, "profile": PROFILE,
                }));
            } else {
                println!("ember {}", env!("CARGO_PKG_VERSION"));
                println!("language: {}", ember_parser::LANGUAGE_VERSION);
                println!("backend: c");
                println!("C compiler: {compiler}");
                println!("linker: {linker}");
                println!("profile: {PROFILE}");
            }
            Ok(ExitCode::SUCCESS)
        }
        "new" => new_package(&args[1..]),
        "update" => packages::update(&args[1..]),
        // `[DIA-8]` — each loan live at a line, from the borrow checker's own loans.
        "explain" if args.get(1).is_some_and(|argument| argument == "--borrow") => {
            let target = args.get(2).ok_or("`ember explain --borrow` needs a place: <file>:<line>")?;
            if let Some(extra) = args.get(3) {
                return Err(format!("unexpected `ember explain --borrow` argument `{extra}`"));
            }
            let (file, line) = target
                .rsplit_once(':')
                .and_then(|(file, line)| Some((file, line.parse::<u32>().ok().filter(|line| *line > 0)?)))
                .ok_or_else(|| format!("`{target}` is not <file>:<line>"))?;
            let options = Options { borrow_report: Some(line), ..Options::default() };
            compile(Path::new(file), "check", &options)
        }
        "explain" if args.get(1).is_some_and(|argument| argument == "--cycle") => {
            let rest: Vec<&String> = args[2..].iter().filter(|argument| *argument != "--json").collect();
            let input = rest
                .first()
                .ok_or("`ember explain --cycle` needs an analysis root path")?;
            let target = rest
                .get(1)
                .ok_or("`ember explain --cycle` needs a class or Class.field target")?;
            if let Some(extra) = rest.get(2) {
                return Err(format!("unexpected cycle explanation argument `{extra}`"));
            }
            explain_cycle(Path::new(input.as_str()), target, json)
        }
        "explain" => {
            let rest: Vec<&String> = args[1..].iter().filter(|argument| *argument != "--json").collect();
            let code = rest
                .first()
                .ok_or("`ember explain` needs a code, e.g. E3040")?;
            if let Some(extra) = rest.get(1) {
                return Err(format!("unexpected `ember explain` argument `{extra}`"));
            }
            explain(code, json)
        }
        "inspect" => inspect(&args[1..]),
        "tcb" => tcb(&args[1..]),
        "audit" => audit(&args[1..]),
        "calls" => calls(&args[1..]),
        "bind" => cimport::bind(&args[1..]),
        "why" => why(&args[1..]),
        "test" => test_runner::command(&args[1..]),
        "bench" => bench::command(&args[1..]),
        "build" | "run" | "check" | "lint" => {
            // `[LNT-6]` — `ember lint` is `ember check` and its own lints.
            let lint = command == "lint";
            let command = if lint { "check" } else { command };
            // The source file may sit before or after the flags. Requiring it
            // first made `ember check --syntax-only f.em` fail with "needs a
            // source file", which is a confusing way to say "wrong order".
            let mut input = None;
            let mut rest = Vec::new();
            let mut program_args = Vec::new();
            let mut skip_value = false;
            for (at, arg) in args[1..].iter().enumerate() {
                if skip_value {
                    rest.push(arg.clone());
                    skip_value = false;
                    continue;
                }
                // XVII.1's `ember run [file.em] [args…]` — after the file, words go to
                // the program, and so does everything after `--`, dashes included.
                if command == "run" && arg == "--" {
                    program_args.extend(args[at + 2..].iter().cloned());
                    break;
                }
                if command == "run" && input.is_some() && !arg.starts_with('-') {
                    program_args.push(arg.clone());
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
            let input = match input {
                Some(input) => input,
                // `[TOOL-4]` — in a package, `ember run` (and `build`, `check`) needs no file: the
                // manifest's entry is the program.
                None => {
                    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
                    let mut candidate = Some(cwd.as_path());
                    let root = loop {
                        let Some(dir) = candidate else {
                            return Err(format!("`ember {command}` needs a source file or a package manifest in this directory or an ancestor"));
                        };
                        if dir.join(ember_branding::MANIFEST).is_file() { break dir.to_path_buf(); }
                        candidate = dir.parent();
                    };
                    root.to_string_lossy().into_owned()
                }
            };
            let mut options = parse_options(&rest)?;
            options.program_args = program_args;
            options.lint = lint;
            if options.leak_check && command != "run" {
                return Err("`--leak-check` is only valid with `ember run` and `ember test`".to_string());
            }
            if options.no_leak_check && command != "run" {
                return Err("`--no-leak-check` is only valid with `ember run` and `ember test`".to_string());
            }
            if options.leak_check && options.no_leak_check {
                return Err("`--leak-check` and `--no-leak-check` contradict each other".to_string());
            }
            // `[WK-15]` — `ember run` in the debug profile reports leaked
            // objects and their cycles at exit unless told not to.
            let mut options = options;
            if command == "run" && options.profile == Profile::Debug && !options.no_leak_check {
                options.leak_check = true;
            }
            if options.emit_header && command != "build" {
                return Err("`--emit-header` is only valid with `ember build`".to_string());
            }
            if options.reload && command != "build" {
                return Err("`--reload` is only valid with `ember build`; `ember run --hot` reloads as it runs".to_string());
            }
            if options.explain && !options.reload {
                return Err("`--explain` is only valid with `ember build --reload`".to_string());
            }
            if options.hot && command != "run" {
                return Err("`--hot` is only valid with `ember run`".to_string());
            }
            if options.emit.as_deref() == Some("header") && command != "build" {
                return Err("`--emit header` is only valid with `ember build`".to_string());
            }
            if options.emit_header && options.emit.as_deref().is_some_and(|stage|
                !matches!(stage, "c" | "header")) {
                return Err("`--emit-header` requires a normal build, `--emit c`, or `--emit header`".to_string());
            }
            let source = if Path::new(&input).is_dir() {
                let root = Path::new(&input);
                let manifest = root.join(ember_branding::MANIFEST);
                if !manifest.is_file() {
                    return Err(format!("package directory `{}` has no {}", root.display(), ember_branding::MANIFEST));
                }
                package_entry_from_manifest(root, &manifest, command)?.0
            } else {
                PathBuf::from(input)
            };
            // `[HR-33]` — the toolchain's own host: build, run, watch, rebuild, reload.
            if command == "run" && options.hot && options.profile != Profile::Release {
                return run_hot(&source, &options);
            }
            compile(&source, command, &options)
        }
        // `[FMT-1]` — the canonical printer. `--check` reports whether the
        // file is already formatted instead of rewriting it.
        "fmt" => {
            let mut input = None;
            for argument in &args[1..] {
                match argument.as_str() {
                    "--check" | "--write" | "--json" | "--migrate" => {}
                    option if option.starts_with('-') => return Err(format!("unknown `ember fmt` option `{option}`")),
                    file if input.is_none() => input = Some(file),
                    extra => return Err(format!("unexpected `ember fmt` argument `{extra}`")),
                }
            }
            let input = input.ok_or("`ember fmt` needs a source file")?;
            let check_only = args.iter().any(|a| a == "--check");
            let write = args.iter().any(|a| a == "--write");
            if args.iter().any(|a| a == "--migrate") {
                if check_only {
                    return Err("`--migrate` and `--check` cannot be combined".to_string());
                }
                return migrate_file(Path::new(input), write, json);
            }
            format_file(Path::new(input), check_only, write, json)
        }
        other => Err(format!("unknown command `{other}`; try `ember --help`")),
    }
}

/// `[TOOL-4]` — `ember new <name> [--lib | --bin | --cdylib]`: a package directory with its
/// manifest (the current language, `[VER-2]`) and a first source: a program printing
/// `hello, world` (a first program needs no `main`, `[CLI-4]`), or a library with one function.
/// An existing directory that is not empty is left alone.
fn new_package(args: &[String]) -> Result<ExitCode, String> {
    let mut name = None;
    let mut kind = "bin";
    for argument in args {
        match argument.as_str() {
            "--bin" => kind = "bin",
            "--lib" => kind = "lib",
            "--cdylib" => kind = "cdylib",
            "--json" => {}
            option if option.starts_with('-') => return Err(format!("unknown `ember new` option `{option}`")),
            word if name.is_none() => name = Some(word),
            extra => return Err(format!("unexpected `ember new` argument `{extra}`")),
        }
    }
    let path = Path::new(name.ok_or("`ember new` needs a package name, e.g. `ember new hello`")?);
    let package = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
    let mut chars = package.chars();
    if !chars.next().is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(format!("`{package}` is not a package name: a letter or `_`, then letters, digits, `_` or `-`"));
    }
    if path.read_dir().is_ok_and(|mut entries| entries.next().is_some()) {
        return Err(format!("`{}` already exists and is not empty", path.display()));
    }
    let (file, source) = match kind {
        "bin" => ("main", "println(\"hello, world\")\n".to_string()),
        "lib" => ("lib", "pub fn answer() -> i32:\n    return 42\n".to_string()),
        _ => ("lib", format!("@export(\"{}_answer\")\nfn answer() -> i32:\n    return 42\n", package.replace('-', "_"))),
    };
    let manifest = format!(
        "[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nlanguage = \"{}\"\nkind = \"{kind}\"\n",
        ember_parser::LANGUAGE_VERSION
    );
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(path.join("src"))?;
        std::fs::write(path.join(ember_branding::MANIFEST), manifest)?;
        std::fs::write(path.join("src").join(ember_branding::source_file(file)), source)
    };
    write().map_err(|error| format!("cannot create `{}`: {error}", path.display()))?;
    eprintln!("created {} package `{package}`", if kind == "bin" { "binary" } else { kind });
    Ok(ExitCode::SUCCESS)
}

/// `[RC-6]` — the side table `ember inspect --counts` reads.
fn counts_json(bodies: &[ember_mir::Body], types: &TypeTable, map: &SourceMap) -> String {
    let operations: Vec<serde_json::Value> = ember_analysis::count_report::surviving_count_operations(bodies, types)
        .into_iter()
        .map(|operation| {
            serde_json::json!({
                "kind": if operation.retain { "retain" } else { "release" },
                "source": map.location(operation.span),
                "function": operation.function,
                "loop": map.location(operation.loop_span),
                "reason": operation.reason,
            })
        })
        .collect();
    format!("{}\n", serde_json::json!({ "schema": 1, "operations": operations }))
}

fn inspect(args: &[String]) -> Result<ExitCode, String> {
    let mut safety = false;
    let mut counts = false;
    let mut cycle = false;
    let mut alloc = false;
    let mut deterministic: Option<String> = None;
    let mut cost = false;
    let mut generics = false;
    let mut elided_only = false;
    let mut json = false;
    let mut function = None;
    let mut path = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--safety" => safety = true,
            "--counts" => counts = true,
            "--cycle" => cycle = true,
            "--alloc" => alloc = true,
            "--cost" => cost = true,
            "--generics" => generics = true,
            "--deterministic" => {
                index += 1;
                let item = args.get(index).filter(|item| !item.starts_with('-'));
                deterministic = Some(item.ok_or("`--deterministic` needs an item: a function the program names")?.clone());
            }
            "--elided-only" => elided_only = true,
            "--json" => json = true,
            "--function" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or("`--function` needs a function name")?;
                if value.starts_with('-') {
                    return Err("`--function` needs a function name".to_string());
                }
                if function.replace(value.clone()).is_some() {
                    return Err("`--function` may be specified only once".to_string());
                }
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown inspect option `{value}`"));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => return Err(format!("unexpected inspect argument `{value}`")),
        }
        index += 1;
    }

    if [safety, counts, cycle, alloc, cost, generics, deterministic.is_some()].iter().filter(|&&kind| kind).count() > 1 {
        return Err("`ember inspect` accepts only one report kind at a time".to_string());
    }
    if !safety && !counts && !cycle && !alloc && !cost && !generics && deterministic.is_none() {
        return Err("`ember inspect` requires `--safety`, `--counts`, `--cycle`, `--alloc`, `--cost`, `--generics` or `--deterministic <item>`".to_string());
    }
    // `[MONO-7]` — checked as `ember check` does.
    if generics {
        if elided_only || function.is_some() || json {
            return Err("`--elided-only`, `--function` and `--json` do not apply to `--generics`".to_string());
        }
        let path = path.ok_or("`ember inspect --generics` needs a source file")?;
        let options = Options { generics_report: true, ..Options::default() };
        return compile(&path, "check", &options);
    }
    // `[COST-4]` — from the safety side table a build wrote, like `--safety`.
    if cost {
        if elided_only {
            return Err("`--elided-only` applies only to `--safety`".to_string());
        }
        let path = path.ok_or("`ember inspect --cost` needs a side-table path")?;
        return inspect_cost(&path, function.as_deref(), json);
    }
    // `[DET-9]` — checked as `ember check` does.
    if let Some(item) = deterministic {
        if elided_only || function.is_some() {
            return Err("`--elided-only` and `--function` apply only to `--safety` and `--alloc`".to_string());
        }
        let path = path.ok_or("`ember inspect --deterministic` needs a source file")?;
        let report = EffectReport { effect: ember_hir::Effects::NONDET, item, json, deterministic: true };
        let options = Options { effect_report: Some(report), ..Options::default() };
        return compile(&path, "check", &options);
    }
    // `[PHIL-2]`, `[CLO-10]` — checked as `ember check` does; diagnostics as
    // text on standard error, as `--cycle`'s are, so `--json` is the report's.
    if alloc {
        if elided_only {
            return Err("`--elided-only` applies only to `--safety`".to_string());
        }
        let path = path.ok_or("`ember inspect --alloc` needs a source file")?;
        let options = Options { alloc_report: Some(AllocReport { json, function }), ..Options::default() };
        return compile(&path, "check", &options);
    }
    if counts {
        if elided_only {
            return Err("`--elided-only` applies only to `--safety`".to_string());
        }
        let path = path.ok_or("`ember inspect --counts` needs a side-table path")?;
        return inspect_counts(&path, function.as_deref(), json);
    }
    if cycle {
        if elided_only || function.is_some() {
            return Err("`--elided-only` and `--function` apply only to `--safety`".to_string());
        }
        let path = path.ok_or("`ember inspect --cycle` needs a source file")?;
        return inspect_cycle(&path, json);
    }
    let path = path.ok_or("`ember inspect --safety` needs a side-table path")?;
    inspect_safety(&path, elided_only, function.as_deref(), json)
}

/// `[CLI-17]` — inspect the exact package-local graph that `[WK-5]` uses.
/// This intentionally ends at type checking: cycle inspection is a
/// diagnostic-only declaration analysis, so lowering and code generation
/// would not add evidence and could obscure the source ownership types.
fn inspect_cycle(input: &Path, json: bool) -> Result<ExitCode, String> {
    cycle_analysis(input, |entry, _, inspection| {
        if json {
            print_cycle_json(inspection)?;
        } else {
            print_cycle_report(entry, inspection);
        }
        Ok(())
    })
}

/// `[CLI-17]` / `[WK-9]` — build one declaration-level ownership graph from
/// the root `[CLI-18]` resolved. Both renderers receive this same graph; no
/// hidden build state or cycle-specific graph representation participates.
fn cycle_analysis(
    input: &Path,
    render: impl FnOnce(
        &Path,
        &TypeTable,
        &ember_analysis::OwnershipInspection,
    ) -> Result<(), String>,
) -> Result<ExitCode, String> {
    let (entry, root_dir) = resolve_cycle_analysis_root(input)?;
    let mut map = SourceMap::new();
    let file = match map.load(&entry) {
        Ok(file) => file,
        Err(error @ LoadError::NotUtf8(_)) => {
            eprintln!("{}", ember_diag::render(&load_diagnostic(&error, Span::DUMMY), &map));
            return Ok(ExitCode::FAILURE);
        }
        Err(error) => return Err(error.to_string()),
    };
    let source = map.file(file).text.clone();
    let mut sink = Sink::new();
    let lexed = ember_lexer::lex(file, &source, &mut sink);
    let module = ember_parser::parse(file, &source, lexed.tokens, &mut sink);
    if sink.has_errors() {
        report(&sink, &map, &Options::default());
        return Ok(ExitCode::FAILURE);
    }

    let lint_settings = manifest_lint_settings(&root_dir, &mut map, &mut sink);
    let modules = load_modules(module, &root_dir, &mut map, &mut sink);
    if sink.has_errors() {
        report(&sink, &map, &Options::default());
        return Ok(ExitCode::FAILURE);
    }

    let (mut types, common) = TypeTable::new();
    ember_typeck::check(
        &modules,
        &mut types,
        &common,
        &mut sink,
        overflow_policy(Profile::Debug),
        lint_settings.return_intersection,
        true,
        &ember_typeck::ComptimeAnswers::new(),
        lint_settings.realtime,
    );
    if sink.has_errors() {
        report(&sink, &map, &Options::default());
        return Ok(ExitCode::FAILURE);
    }

    let inspection = ember_analysis::inspect_ownership_graph(&types);
    report(&sink, &map, &Options::default());
    render(&entry, &types, &inspection)?;
    Ok(ExitCode::SUCCESS)
}

/// `[CLI-18]` — resolve the one explicit analysis root accepted by both cycle
/// commands. A source file always keeps the existing standalone interpretation;
/// it is never silently promoted to the package that may contain it.
fn resolve_cycle_analysis_root(input: &Path) -> Result<(PathBuf, PathBuf), String> {
    if input.is_file() {
        if input.extension().and_then(|extension| extension.to_str())
            != Some(ember_branding::SOURCE_EXT)
        {
            return Err(format!(
                "cycle analysis root `{}` must be a package directory or a .{} source file",
                input.display(),
                ember_branding::SOURCE_EXT
            ));
        }
        let module_root = input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        return Ok((input.to_path_buf(), module_root));
    }

    if input.is_dir() {
        let manifest = input.join(ember_branding::MANIFEST);
        if !manifest.is_file() {
            return Err(format!(
                "cycle analysis root `{}` is not a package directory containing {}",
                input.display(),
                ember_branding::MANIFEST
            ));
        }
        return package_entry_from_manifest(input, &manifest, "cycle analysis");
    }

    Err(format!(
        "cycle analysis root `{}` must be an existing package directory or .{} source file",
        input.display(),
        ember_branding::SOURCE_EXT
    ))
}

fn package_entry_from_manifest(
    package_root: &Path,
    manifest_path: &Path,
    purpose: &str,
) -> Result<(PathBuf, PathBuf), String> {
    let manifest = std::fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "cannot read {purpose} manifest `{}`: {error}",
            manifest_path.display()
        )
    })?;
    let source_root = package_root.join("src");
    let entry = manifest_string(&manifest, "build", "entry").map_or_else(
        || {
            let root_name = if matches!(manifest_string(&manifest, "package", "kind").as_deref(),
                Some("lib" | "staticlib" | "cdylib"))
            {
                "lib"
            } else {
                "main"
            };
            source_root.join(ember_branding::source_file(root_name))
        },
        |path| package_root.join(path),
    );

    if !entry.is_file() {
        return Err(format!(
            "{purpose} package `{}` has no readable entry source `{}`",
            package_root.display(),
            entry.display()
        ));
    }
    if entry.extension().and_then(|extension| extension.to_str())
        != Some(ember_branding::SOURCE_EXT)
    {
        return Err(format!(
            "{purpose} package `{}` entry `{}` is not a .{} source file",
            package_root.display(),
            entry.display(),
            ember_branding::SOURCE_EXT
        ));
    }
    let inside_source = std::fs::canonicalize(&entry).ok().zip(std::fs::canonicalize(&source_root).ok())
        .is_some_and(|(entry, root)| entry.starts_with(root));
    if !inside_source {
        return Err(format!(
            "{purpose} package `{}` entry `{}` is outside its src directory",
            package_root.display(),
            entry.display()
        ));
    }
    Ok((entry, source_root))
}

/// Read the one quoted manifest scalar that root selection needs. Full manifest
/// validation is outside cycle-root selection; this helper only distinguishes
/// the established package root forms before loading a graph.
fn manifest_string(manifest: &str, wanted_section: &str, wanted_key: &str) -> Option<String> {
    let mut section = "";
    for line in manifest.lines() {
        let line = line.split_once('#').map_or(line, |(before, _)| before).trim();
        if let Some(name) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            section = name.trim();
            continue;
        }
        if section != wanted_section {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != wanted_key {
            continue;
        }
        let value = value.trim();
        return value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(str::to_owned);
    }
    None
}

/// A manifest list of quoted strings, written on one line (`libs = ["m", "vulkan-1"]`).
fn manifest_list(manifest: &str, wanted_section: &str, wanted_key: &str) -> Option<Vec<String>> {
    let mut section = "";
    for line in manifest.lines() {
        let line = line.split_once('#').map_or(line, |(before, _)| before).trim();
        if let Some(name) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            section = name.trim();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        if section != wanted_section || key.trim() != wanted_key {
            continue;
        }
        let items = value.trim().strip_prefix('[')?.strip_suffix(']')?;
        return Some(items.split(',').map(|item| item.trim().trim_matches('"').to_string()).filter(|item| !item.is_empty()).collect());
    }
    None
}

/// `[WK-9]` — select one already-resolved class or field from the ownership
/// graph. This does not add a name system: source-qualified names are the
/// type table's existing dotted canonical names, written at the CLI with `::`.
fn explain_cycle(input: &Path, target: &str, json: bool) -> Result<ExitCode, String> {
    cycle_analysis(input, |entry, types, inspection| {
        let (class, field) = resolve_cycle_target(entry, types, target)?;
        let selected_edges: Vec<_> = inspection
            .edges
            .iter()
            .filter(|edge| {
                edge.source == class
                    && field.as_deref().is_none_or(|field| edge.field == field)
            })
            .collect();
        let paths: Vec<_> = inspection
            .shortest_cycles
            .iter()
            .filter(|cycle| {
                cycle.edges.iter().any(|edge| {
                    edge.source == class
                        && field.as_deref().is_none_or(|field| edge.field == field)
                })
            })
            .collect();

        let selected = field
            .as_deref()
            .map_or_else(
                || class.clone(),
                |field| format!("{}.{}", class, field),
            );
        if json {
            let edges: Vec<_> = selected_edges
                .iter()
                .map(|edge| {
                    serde_json::json!({
                        "source": edge.source,
                        "field": edge.field,
                        "type": edge.field_type,
                        "target": edge.target,
                        "kind": edge.kind.name(),
                    })
                })
                .collect();
            let paths: Vec<_> = paths.iter().map(|path| cycle_explanation_path(&path.edges)).collect();
            println!("{}", serde_json::json!({ "schema": 1, "target": selected, "edges": edges, "static_paths": paths }));
            return Ok(());
        }
        println!("Cycle explanation: {selected}");
        println!("Analysis root: {}", entry.display());
        println!("Selected ownership declarations:");
        if selected_edges.is_empty() {
            println!("  none");
        } else {
            for edge in selected_edges {
                print_cycle_explanation_edge(edge);
            }
        }
        println!("Static ownership paths:");
        if paths.is_empty() {
            println!("  none");
            if field.is_some() {
                println!(
                    "The selected edge is dynamically cycle-capable; no static cycle was proved."
                );
            }
            return Ok(());
        }
        for path in paths {
            println!("  {}", cycle_explanation_path(&path.edges));
        }
        Ok(())
    })
}

/// `[WK-9]`, `[GRM-24]` (0.9.9) — `<Class[.field]>`, with `.` the one path
/// separator. A path resolves left to right, so the whole target names a
/// class, or all of it but a last segment that names the field.
fn resolve_cycle_target(
    entry: &Path,
    types: &TypeTable,
    target: &str,
) -> Result<(String, Option<String>), String> {
    if target.contains("::") {
        return Err(format!(
            "invalid cycle target `{target}` for analysis root `{}`: use '.' for paths",
            entry.display()
        ));
    }
    if target.split('.').any(str::is_empty) {
        return Err(format!(
            "invalid cycle target `{target}` for analysis root `{}`",
            entry.display()
        ));
    }
    let classes_named = |written: &str| -> Vec<_> {
        types
            .classes()
            .filter(|(_, definition)| {
                let name = definition.name.as_str();
                name == written
                    || (!written.contains('.') && name.rsplit('.').next() == Some(written))
            })
            .collect()
    };
    // A prefix that is a module (it qualifies some class) is not a class, so
    // what is missing is the class the whole target names inside it.
    let is_module = |written: &str| {
        let prefix = format!("{written}.");
        types.classes().any(|(_, definition)| definition.name.as_str().starts_with(&prefix))
    };
    let (written_class, field, candidates) = match (classes_named(target), target.rsplit_once('.')) {
        (whole, _) if !whole.is_empty() => (target, None, whole),
        (whole, Some((class, _))) if classes_named(class).is_empty() && is_module(class) => {
            (target, None, whole)
        }
        (_, Some((class, field))) => (class, Some(field), classes_named(class)),
        (whole, None) => (target, None, whole),
    };
    let Some((class_id, definition)) = candidates.first().copied() else {
        return Err(format!(
            "cannot find class `{written_class}` in cycle analysis root `{}`",
            entry.display()
        ));
    };
    if candidates.len() > 1 {
        let candidates = candidates
            .iter()
            .map(|(_, definition)| definition.name.as_str().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "class `{written_class}` is ambiguous in cycle analysis root `{}`; candidates: {candidates}",
            entry.display()
        ));
    }
    let class = definition.name.to_string();
    let field = field.map(str::to_owned);
    if let Some(field_name) = field.as_deref() {
        let exists = (0..types.class_field_count(class_id)).any(|index| {
            types
                .class_field_at_info(class_id, index)
                .is_some_and(|(_, field)| field.name.as_str() == field_name)
        });
        if !exists {
            return Err(format!(
                "class `{}` has no field `{field_name}` in cycle analysis root `{}`",
                class,
                entry.display()
            ));
        }
    }
    Ok((class, field))
}

fn print_cycle_edge(edge: &ember_analysis::OwnershipEdge) {
    let target = edge.target.as_deref().unwrap_or("<unknown>");
    println!(
        "  {} {}.{}: {} -> {}",
        edge.kind.name(),
        edge.source,
        edge.field,
        edge.field_type,
        target
    );
}

fn print_cycle_explanation_edge(edge: &ember_analysis::OwnershipEdge) {
    let target = edge
        .target
        .as_deref()
        .map(str::to_string)
        .unwrap_or_else(|| "<unknown>".to_string());
    println!(
        "  {} {}.{}: {} -> {}",
        edge.kind.name(),
        edge.source,
        edge.field,
        edge.field_type,
        target
    );
}

fn cycle_path(edges: &[ember_analysis::OwnershipEdge]) -> String {
    let Some(first) = edges.first() else {
        return String::new();
    };
    edges
        .iter()
        .map(|edge| format!("{}.{}", edge.source, edge.field))
        .chain(std::iter::once(first.source.clone()))
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn cycle_explanation_path(edges: &[ember_analysis::OwnershipEdge]) -> String {
    let Some(first) = edges.first() else {
        return String::new();
    };
    edges
        .iter()
        .map(|edge| format!("{}.{}", edge.source, edge.field))
        .chain(std::iter::once(first.source.clone()))
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn print_cycle_report(path: &Path, inspection: &ember_analysis::OwnershipInspection) {
    println!("Ownership graph: {}", path.display());
    println!("Edges:");
    if inspection.edges.is_empty() {
        println!("  none");
    }
    for edge in &inspection.edges {
        print_cycle_edge(edge);
    }
    println!("Shortest static cycles:");
    if inspection.shortest_cycles.is_empty() {
        println!("  none");
        return;
    }
    for cycle in &inspection.shortest_cycles {
        println!("  {}", cycle_path(&cycle.edges));
    }
}

fn print_cycle_json(inspection: &ember_analysis::OwnershipInspection) -> Result<(), String> {
    let edges: Vec<_> = inspection
        .edges
        .iter()
        .map(|edge| {
            serde_json::json!({
                "source": edge.source,
                "declaration": edge.declaration,
                "field": edge.field,
                "type": edge.field_type,
                "target": edge.target,
                "kind": edge.kind.name(),
            })
        })
        .collect();
    let cycles: Vec<_> = inspection
        .shortest_cycles
        .iter()
        .map(|cycle| {
            cycle
                .edges
                .iter()
                .map(|edge| format!("{}.{}", edge.source, edge.field))
                .collect::<Vec<_>>()
        })
        .collect();
    let report = serde_json::json!({
        "schema": 1,
        "edges": edges,
        "shortest_cycles": cycles,
    });
    println!(
        "{}",
        serde_json::to_string(&report).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn inspect_safety(
    path: &Path,
    elided_only: bool,
    function: Option<&str>,
    json: bool,
) -> Result<ExitCode, String> {
    let resolved = resolve_safety_path(path);
    let text = std::fs::read_to_string(&resolved).map_err(|error| {
        format!(
            "could not read safety side table `{}`: {error}",
            resolved.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
        format!(
            "invalid safety side table `{}`: {error}",
            resolved.display()
        )
    })?;
    let schema = value
        .get("schema")
        .and_then(serde_json::Value::as_u64)
        .ok_or("safety side table is missing numeric schema")?;
    if schema != 1 {
        return Err(format!("unsupported safety side-table schema {schema}"));
    }
    let checks = value
        .get("checks")
        .and_then(serde_json::Value::as_array)
        .ok_or("safety side table is missing its checks array")?;
    for check in checks {
        validate_safety_check(check)?;
    }
    let selected: Vec<serde_json::Value> = checks
        .iter()
        .filter(|check| {
            (!elided_only
                || check.get("status").and_then(serde_json::Value::as_str) == Some("elided"))
                && function.is_none_or(|name| {
                    check.get("function").and_then(serde_json::Value::as_str) == Some(name)
                })
        })
        .cloned()
        .collect();

    // `[EFF-16]` — written since the functions were added to schema 1; an older table has none.
    let functions: Vec<serde_json::Value> = value
        .get("functions")
        .and_then(serde_json::Value::as_array)
        .map(|all| {
            all.iter()
                .filter(|entry| function.is_none_or(|name| entry.get("function").and_then(serde_json::Value::as_str) == Some(name)))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    if json {
        let report = serde_json::json!({ "schema": schema, "checks": selected, "functions": functions });
        println!(
            "{}",
            serde_json::to_string(&report).map_err(|error| error.to_string())?
        );
    } else {
        print_safety_report(&resolved, &selected, elided_only, function);
        print_safety_functions(&functions);
    }
    Ok(ExitCode::SUCCESS)
}

/// `[COST-3]` — a check's row of the cost table, by its kind.
/// `[COST-5]` — `tools/rule_index.py` reads the rows named here and in `safety_costs_json`'s `add`
/// calls, and fails CI on one the table of `[COST-3]` does not have: name a new row with a literal.
fn check_cost_row<'a>(kind: &'a str, mechanism: &str) -> &'a str {
    match (kind, mechanism) {
        ("Bounds", _) => "Bounds check on `a[i]`",
        ("Overflow", _) => "Overflow check on integer arithmetic",
        ("DivisionByZero" | "ShiftRange", _) => "Division-by-zero and shift-range checks",
        ("Aliasing", "assert_disjoint") => "`assert_disjoint` comparison",
        ("Aliasing", _) => "Dynamic exclusivity check",
        ("Borrow", _) => "`RefCell` borrow",
        ("Downcast", _) => "Downcast check",
        // A kind this compiler does not emit (a side table from another version): shown as named.
        _ => kind,
    }
}

/// `[COST-4]` — `ember inspect <table> --cost [--function <f>] [--json]`: every row of the cost table
/// (`[COST-3]`) that applies, with its class (`[COST-2]`), each site under it; for a conditionally
/// elided cost, whether each was removed and by which proof.
fn inspect_cost(path: &Path, function: Option<&str>, json: bool) -> Result<ExitCode, String> {
    let resolved = resolve_side_table(path, "safety");
    let text = std::fs::read_to_string(&resolved).map_err(|error| format!("cannot read safety side table `{}`: {error}", resolved.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| format!("invalid safety side table `{}`: {error}", resolved.display()))?;
    let field = |entry: &serde_json::Value, name: &str| entry.get(name).and_then(serde_json::Value::as_str).unwrap_or("").to_string();
    // (row, class, source, function, what)
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    for check in value.get("checks").and_then(serde_json::Value::as_array).into_iter().flatten() {
        let (kind, mechanism, status) = (field(check, "kind"), field(check, "mechanism"), field(check, "status"));
        let row = check_cost_row(&kind, &mechanism);
        let class = match row {
            "`assert_disjoint` comparison" | "`RefCell` borrow" | "Downcast check" => "guaranteed present",
            _ => "conditionally elided",
        };
        let what = if status == "elided" || check.get("classification").and_then(serde_json::Value::as_str) == Some("DYNAMIC_HOISTED_LOOP") {
            let proof = check.get("proof").and_then(serde_json::Value::as_str).map(str::to_string).unwrap_or_else(|| field(check, "reason"));
            format!("removed: {mechanism} ({proof})")
        } else {
            format!("present: {mechanism}")
        };
        rows.push((row.to_string(), class.to_string(), field(check, "source"), field(check, "function"), what));
    }
    for entry in value.get("costs").and_then(serde_json::Value::as_array).into_iter().flatten() {
        rows.push((field(entry, "row"), field(entry, "class"), field(entry, "source"), field(entry, "function"), field(entry, "detail")));
    }
    // The program's own sites: the standard library's are its own (`[COST-4]` reports an item's).
    let normal = |path: &str| path.replace('\\', "/").to_lowercase();
    let std_root = ember_branding::std_root().map(|root| normal(&root.display().to_string()));
    rows.retain(|(_, _, source, owner, _)| {
        std_root.as_ref().is_none_or(|root| !normal(source).starts_with(root.as_str()))
            && function.is_none_or(|name| owner == name || owner.is_empty())
    });
    let mut order: Vec<String> = Vec::new();
    for (row, ..) in &rows {
        if !order.contains(row) {
            order.push(row.clone());
        }
    }
    if json {
        let grouped: Vec<serde_json::Value> = order
            .iter()
            .map(|row| {
                let sites: Vec<&(String, String, String, String, String)> = rows.iter().filter(|entry| &entry.0 == row).collect();
                serde_json::json!({
                    "row": row,
                    "class": sites[0].1,
                    "sites": sites.iter().map(|site| serde_json::json!({ "source": site.2, "function": site.3, "what": site.4 })).collect::<Vec<_>>(),
                })
            })
            .collect();
        println!("{}", serde_json::json!({ "schema": 1, "costs": grouped }));
        return Ok(ExitCode::SUCCESS);
    }
    println!("Costs: {}", resolved.display());
    if let Some(function) = function {
        println!("Function: {function}");
    }
    if order.is_empty() {
        println!("  none");
    }
    for row in &order {
        let sites: Vec<&(String, String, String, String, String)> = rows.iter().filter(|entry| &entry.0 == row).collect();
        println!("  {row} [{}]", sites[0].1);
        for (_, _, source, owner, what) in sites {
            if owner.is_empty() {
                println!("    {source}  {what}");
            } else {
                println!("    {source}  {owner}  {what}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// `[UNS-8]` — `ember tcb [--module <m>] [--json] <file.em>`.
fn tcb(args: &[String]) -> Result<ExitCode, String> {
    let mut request = TcbReport::default();
    let mut require = false;
    let mut path = None;
    let mut rest = args.iter();
    while let Some(argument) = rest.next() {
        match argument.as_str() {
            "--json" => request.json = true,
            "--module" => request.module = Some(rest.next().ok_or("`--module` needs a module name")?.clone()),
            "--require" => require = true,
            flag if flag.starts_with('-') => return Err(format!("unknown `ember tcb` option `{flag}`")),
            file if path.is_none() => path = Some(PathBuf::from(file)),
            extra => return Err(format!("unexpected `ember tcb` argument `{extra}`")),
        }
    }
    let path = path.ok_or("`ember tcb` needs a source file")?;
    let options = Options { tcb_report: Some(request), require_evidence: require, ..Options::default() };
    compile(&path, "check", &options)
}

/// `[CLI-11]` — `ember audit [--json] <file.em>`: a one-page summary of a package's safety surface.
fn audit(args: &[String]) -> Result<ExitCode, String> {
    let mut request = TcbReport { audit: true, ..TcbReport::default() };
    let mut path = None;
    for argument in args {
        match argument.as_str() {
            "--json" => request.json = true,
            flag if flag.starts_with('-') => return Err(format!("unknown `ember audit` option `{flag}`")),
            file if path.is_none() => path = Some(PathBuf::from(file)),
            extra => return Err(format!("unexpected `ember audit` argument `{extra}`")),
        }
    }
    let path = path.ok_or("`ember audit` needs a source file")?;
    let options = Options { tcb_report: Some(request), ..Options::default() };
    compile(&path, "check", &options)
}

/// `[TCB-4]` — the defects the compiler's own ledger (`docs/DEFECTS.md`) lists as open, by number.
fn known_defects() -> Vec<&'static str> {
    const LEDGER: &str = include_str!("../../../docs/DEFECTS.md");
    LEDGER
        .lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            // `| D-NNN | what | rules | status | … |`
            let (id, status) = (cells.get(1)?, cells.get(4)?);
            (id.starts_with("D-") && status.starts_with("**open")).then_some(*id)
        })
        .collect()
}

/// `[TCB-4]`, `[FFI-3]` — the C++ the program trusts: the thunks of each header it imports, which
/// the C++ compiler builds with the project's flags, and `std.ffi`'s bridge types (`[FFI-17e]`).
fn cpp_bridge(modules: &[ember_typeck::LoadedModule]) -> String {
    let mut headers: Vec<&str> = modules
        .iter()
        .filter_map(|loaded| loaded.foreign.as_ref().filter(|foreign| foreign.cpp.is_some() && !foreign.header.is_empty()).map(|foreign| foreign.header.as_str()))
        .collect();
    headers.sort_unstable();
    headers.dedup();
    if headers.is_empty() {
        return "none: the program imports no C++".to_string();
    }
    format!(
        "the `extern \"C\"` thunks of {}, compiled by the C++ compiler with the project's flags, and `std.ffi`'s bridge types (`CppVector`, `CppString`, `CppShared`) over the C++ standard library",
        headers.iter().map(|header| format!("`{header}`")).collect::<Vec<_>>().join(", ")
    )
}

/// `[TCB-4]` — the trusted base by category, the compiler first, with its version and known defects.
fn trusted_base_categories(modules: &[ember_typeck::LoadedModule]) -> Vec<(&'static str, String)> {
    let requested = std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty());
    let driver = match Toolchain::detect(requested.as_deref()) {
        Ok(toolchain) => format!("{} ({}) and {}", toolchain.name(), toolchain.compiler_path().display(), toolchain.linker()),
        Err(error) => format!("no C compiler found: {error}"),
    };
    let defects = known_defects();
    vec![
        ("compiler", format!(
            "ember {} (language {}, backend c); known defects: {}",
            env!("CARGO_PKG_VERSION"),
            ember_parser::LANGUAGE_VERSION,
            if defects.is_empty() { "none".to_string() } else { defects.join(", ") },
        )),
        ("runtime", format!("{}_rt, the runtime built with this compiler", ember_branding::SYMBOL_PREFIX)),
        ("standard library", format!("{}, as shipped with this compiler", ember_branding::STD_PACKAGE)),
        ("C++ bridge", cpp_bridge(modules)),
        ("external libraries", "the foreign declarations below, each linked from its library".to_string()),
        ("driver", driver),
        ("hardware", format!("{} ({})", std::env::consts::ARCH, std::env::consts::OS)),
    ]
}

/// `[CLI-13]` — `ember calls --foreign <item> [--json] <file.em>`.
fn calls(args: &[String]) -> Result<ExitCode, String> {
    let mut item = None;
    let mut json = false;
    let mut path = None;
    let mut rest = args.iter();
    while let Some(argument) = rest.next() {
        match argument.as_str() {
            "--json" => json = true,
            "--foreign" => item = Some(rest.next().ok_or("`--foreign` needs an item: a function the program names")?.clone()),
            flag if flag.starts_with('-') => return Err(format!("unknown `ember calls` option `{flag}`")),
            file if path.is_none() => path = Some(PathBuf::from(file)),
            extra => return Err(format!("unexpected `ember calls` argument `{extra}`")),
        }
    }
    let item = item.ok_or("`ember calls` needs `--foreign <item>`")?;
    let path = path.ok_or("`ember calls` needs a source file")?;
    let options = Options { foreign_calls: Some(ForeignCalls { json, item }), ..Options::default() };
    compile(&path, "check", &options)
}

/// `[CLI-12]` — `ember why --alloc|--block|--io|--lock|--sync|--unsafe|--ffi <item> <file>`.
fn why(args: &[String]) -> Result<ExitCode, String> {
    use ember_hir::Effects;
    let mut wanted: Option<(Effects, String)> = None;
    let mut json = false;
    let mut path = None;
    let mut rest = args.iter();
    while let Some(argument) = rest.next() {
        let effect = match argument.as_str() {
            "--json" => {
                json = true;
                continue;
            }
            "--alloc" => Effects::ALLOC,
            "--block" => Effects::BLOCK,
            "--io" => Effects::IO,
            "--lock" => Effects::LOCK,
            "--sync" => Effects::SYNC,
            "--unsafe" => Effects::UNSAFE,
            "--ffi" => Effects::FFI,
            flag if flag.starts_with('-') => return Err(format!("unknown `ember why` option `{flag}`")),
            file if path.is_none() => {
                path = Some(PathBuf::from(file));
                continue;
            }
            extra => return Err(format!("unexpected `ember why` argument `{extra}`")),
        };
        let item = rest.next().ok_or_else(|| format!("`{argument}` needs an item: a function the program names"))?;
        if wanted.replace((effect, item.clone())).is_some() {
            return Err("`ember why` takes one effect at a time".to_string());
        }
    }
    let (effect, item) =
        wanted.ok_or("`ember why` needs an effect and an item: `--alloc|--block|--io|--lock|--sync|--unsafe|--ffi <item>`")?;
    let path = path.ok_or("`ember why` needs a source file")?;
    let options = Options { effect_report: Some(EffectReport { effect, item, json, deterministic: false }), ..Options::default() };
    compile(&path, "check", &options)
}

/// The bodies of the standard library, and its files: what a report about the program leaves out.
fn standard_parts(bodies: &[ember_mir::Body], modules: &[ember_typeck::LoadedModule]) -> (Vec<bool>, std::collections::BTreeSet<ember_span::FileId>) {
    let standard_files: std::collections::BTreeSet<ember_span::FileId> = modules
        .iter()
        .filter(|loaded| loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE))
        .map(|loaded| loaded.module.span.file)
        .collect();
    let std_prefix = ember_branding::mangled(&format!("{}.", ember_branding::STD_PACKAGE));
    let standard = bodies
        .iter()
        .map(|body| body.symbol.starts_with(&std_prefix) || standard_files.contains(&body.span.file))
        .collect();
    (standard, standard_files)
}

/// `[CLI-10]` — `ember build --report=engine`: a report, not a profile. It changes nothing about
/// the program; it summarises, for the program's own code (not the standard library's), the
/// allocations, the count operations that survive in loops and why, the runtime checks kept and
/// why, the loops the C compiler vectorised and why it did not others, what each foreign
/// function's wrapper does, the inlining decisions, and the classes that could be structs
/// (advisory, never applied).
#[derive(Default)]
struct EngineReport {
    allocations: Vec<String>,
    inlined: Vec<(String, String, String)>,
    counts: Vec<String>,
    checks: Vec<String>,
    removed_checks: usize,
    loops: Vec<String>,
    wrappers: Vec<String>,
    hints: Vec<String>,
    classes: Vec<String>,
    standard_symbols: std::collections::HashSet<String>,
}

impl EngineReport {
    fn gather_final(&mut self, bodies: &[ember_mir::Body], types: &TypeTable, modules: &[ember_typeck::LoadedModule], map: &SourceMap) {
        let (standard, _) = standard_parts(bodies, modules);
        self.standard_symbols.extend(bodies.iter().zip(&standard).filter(|(_, standard)| **standard).map(|(body, _)| body.symbol.clone()));
        let own: std::collections::HashSet<&str> = bodies.iter().zip(&standard).filter(|(_, standard)| !**standard).map(|(body, _)| body.name.as_str()).collect();
        for operation in ember_analysis::count_report::surviving_count_operations(bodies, types) {
            if own.contains(operation.function.as_str()) {
                let what = if operation.retain { "a retain" } else { "a release" };
                self.counts.push(format!("{} in {}: {what} ({})", map.location(operation.span), operation.function, operation.reason));
            }
        }
        for (body, standard) in bodies.iter().zip(&standard) {
            if *standard {
                continue;
            }
            if let Some(counted) = &body.ffi_counted {
                let mut work: Vec<String> = Vec::new();
                for param in &counted.abi_params {
                    work.push(match param {
                        ember_mir::FfiAbiParam::Value { .. } => continue,
                        ember_mir::FfiAbiParam::SpanPointer { nullable, .. } =>
                            format!("a view as its pointer{}", if *nullable { ", `None` as null" } else { "" }),
                        ember_mir::FfiAbiParam::ArrayPointer { .. } => "a fixed array as its pointer".to_string(),
                        ember_mir::FfiAbiParam::Count { sources, inout, .. } => format!(
                            "a length checked to fit{}{}",
                            if sources.len() > 1 { ", the views' lengths compared" } else { "" },
                            if *inout { ", passed by address and returned" } else { "" },
                        ),
                        ember_mir::FfiAbiParam::Owned { .. } => "a foreign box handed over".to_string(),
                        ember_mir::FfiAbiParam::Retained { .. } => "a retained object's address".to_string(),
                        ember_mir::FfiAbiParam::Out { .. } => "an `out` local zeroed and returned".to_string(),
                    });
                }
                if counted.result_span.is_some() {
                    work.push("the result as a view".to_string());
                }
                if counted.result_owned.is_some() {
                    work.push("the result adopted into a foreign box".to_string());
                }
                if let Some(outputs) = &counted.outputs {
                    work.push(if outputs.status_ok.is_some() { "the status as a `Result`".to_string() } else { "the outputs as the result".to_string() });
                }
                let symbol = ember_branding::export_alias_symbol(&counted.foreign_symbol).unwrap_or_else(|| counted.foreign_symbol.clone());
                self.wrappers.push(format!("{} (`{symbol}`): one call, plus {}", body.name, work.join("; ")));
            } else if body.contracts.foreign.is_some() && body.is_extern_declaration {
                self.wrappers.push(format!("{}: a direct call, no wrapper", body.name));
            }
            let hint = &body.inline;
            for (set, word) in [(hint.always, "`@inline`: always inlined"), (hint.never, "`@noinline`: never inlined"), (hint.cold, "`@cold`: a hint to the C compiler"), (hint.hot, "`@hot`: a hint to the C compiler")] {
                if set {
                    self.hints.push(format!("{}: {word}", body.name));
                }
            }
        }
        // A final class with no base, no virtual method, and no handle of it ever retained: each
        // object has one owner for its whole life, as a struct value does.
        let retained = ember_analysis::count_report::retained_classes(bodies, types);
        let bases: std::collections::HashSet<ember_types::ClassId> = types.classes().filter_map(|(_, def)| def.base).collect();
        let virtual_owners: std::collections::HashSet<ember_types::ClassId> =
            bodies.iter().filter(|body| body.class_virtual_slot.is_some()).filter_map(|body| body.class_owner).collect();
        let standard_files: std::collections::BTreeSet<ember_span::FileId> = standard_parts(bodies, modules).1;
        for (id, def) in types.classes() {
            if def.origin.is_some() || standard_files.contains(&def.span.file) || def.span.is_dummy() {
                continue;
            }
            if def.openness == ember_types::ClassOpenness::Final && def.base.is_none() && !bases.contains(&id)
                && !virtual_owners.contains(&id) && !retained.contains(&id)
            {
                self.classes.push(format!("{} ({}): final, with no base and no virtual method, and no handle of it is ever copied", def.name, map.location(def.span)));
            }
        }
    }

    fn gather_checks(&mut self, safety_json: &str) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(safety_json) else { return };
        let entries = match value {
            serde_json::Value::Array(entries) => entries,
            mut table => table["checks"].as_array_mut().map(std::mem::take).unwrap_or_default(),
        };
        let std_files = format!("{}/", ember_branding::STD_PACKAGE);
        for entry in entries {
            let text = |key: &str| entry[key].as_str().unwrap_or("").to_string();
            // `[BLD-13]` — the standard library's files are `std/…` wherever it is.
            if text("source").starts_with(&std_files) {
                continue;
            }
            if text("status") == "emitted" {
                self.checks.push(format!("{} in {}: {} ({}): {}", text("source"), text("function"), text("kind"), text("mechanism"), text("reason")));
            } else {
                self.removed_checks += 1;
            }
        }
    }

    fn print(&self, toolchain: &Toolchain) {
        println!("engine report (a report: nothing in it changes the program)");
        let section = |title: String, lines: &[String]| {
            println!("{title}");
            if lines.is_empty() {
                println!("  none");
            }
            for line in lines {
                println!("  {line}");
            }
        };
        section(format!("allocations ({}):", self.allocations.len()), &self.allocations);
        section(format!("count operations surviving in loops ({}):", self.counts.len()), &self.counts);
        section(format!("runtime checks kept ({}; {} removed by a proof):", self.checks.len(), self.removed_checks), &self.checks);
        section(format!("loops (the C compiler's vectorisation remarks, {}):", toolchain.name()), &self.loops);
        section(format!("foreign function wrappers ({}):", self.wrappers.len()), &self.wrappers);
        let inlining: Vec<String> = self
            .inlined
            .iter()
            .filter(|(_, _, caller)| !self.standard_symbols.contains(caller))
            .map(|(callee, caller, _)| format!("{callee} inlined into {caller}: called from that one place"))
            .chain(self.hints.iter().cloned())
            .collect();
        section("inlining decisions:".to_string(), &inlining);
        section(format!("classes that could be structs (advisory, never applied; {}):", self.classes.len()), &self.classes);
    }
}

/// `[DIA-8]` — `ember explain --borrow <file>:<line>`: for each loan live at a point of that line of
/// `file`, where it was created, the last line its region covers and the later use that keeps it
/// alive: `borrow of v created at 12:9, live through 19, because s is used at 19:14`.
fn print_borrow_report(map: &SourceMap, file: ember_span::FileId, line: u32, bodies: &[ember_mir::Body], types: &TypeTable) {
    let on_line = |span: &Span| span.file == file && map.line_col(*span).line == line;
    let mut printed = Vec::new();
    for loan in ember_analysis::explain_loans(bodies, types) {
        // Live at the line: its region covers a point there, or it is created there to be used later.
        if loan.created.file != file || !(loan.live.iter().any(on_line) || (on_line(&loan.created) && !loan.live.is_empty())) {
            continue;
        }
        let created = map.line_col(loan.created);
        let through = loan.live.iter().filter(|span| span.file == file).map(|span| map.line_col(*span).line).max().unwrap_or(created.line);
        let kind = if loan.mutable { "mutable borrow" } else { "borrow" };
        let mut text = format!("{kind} of {} created at {}:{}, live through {through}", loan.place, created.line, created.col);
        if let Some((name, span)) = &loan.keeper {
            let used = map.line_col(*span);
            text.push_str(&format!(", because {name} is used at {}:{}", used.line, used.col));
        }
        if !printed.contains(&text) {
            println!("{text}");
            printed.push(text);
        }
    }
    if printed.is_empty() {
        println!("no borrow is live at line {line}");
    }
}

/// `[CLI-12]`, `[DET-9]` — the chains `request` asks for. For `--unsafe`, the obligation of every
/// unsafe function on the chain (`@safety`, `[UNS-7]`); for `--deterministic`, the foreign
/// functions reached whose determinism is asserted (`@ffi(deterministic)`), never verified.
fn print_effect_report(
    input: &Path,
    bodies: &[ember_mir::Body],
    types: &TypeTable,
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
    request: &EffectReport,
) -> Result<(), String> {
    use ember_hir::Effects;
    let effect = if request.deterministic { Effects::NONDET } else { request.effect };
    let word = Effects::NAMED.iter().find(|(named, _)| *named == effect).map_or("?", |(_, word)| *word);
    let roots = ember_analysis::effects::effect_sources(bodies, types, &request.item, effect)
        .ok_or_else(|| format!("no function named `{}` in {}", request.item, input.display()))?;
    let obligations = function_obligations(modules);
    let obligation_of = |name: &str| -> Option<Option<String>> {
        let body = bodies.iter().find(|body| body.is_unsafe && ember_analysis::effects::function_name(body, types) == name)?;
        Some(
            obligations
                .iter()
                .filter(|(span, _)| span.file == body.span.file && span.start <= body.span.start && body.span.start < span.end.max(span.start + 1))
                .min_by_key(|(span, _)| span.end - span.start)
                .and_then(|(_, obligation)| obligation.clone()),
        )
    };
    // `[DET-9]` — determinism a foreign declaration asserts is reported as asserted.
    let asserted: Vec<Vec<String>> = if request.deterministic {
        ember_analysis::effects::foreign_reach(bodies, types, &request.item)
            .map(|reach| {
                reach
                    .reached
                    .into_iter()
                    .filter(|(index, _)| bodies[*index].contracts.foreign.is_some_and(|facts| !facts.contains(Effects::NONDET)))
                    .map(|(_, chain)| chain)
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let (_, sources) = &roots[0];
    let shown: &[(ember_span::Span, Vec<String>)] = if request.deterministic { sources } else { &sources[..sources.len().min(1)] };
    if request.json {
        let chains: Vec<serde_json::Value> = shown
            .iter()
            .map(|(span, chain)| {
                let mut entry = serde_json::json!({ "chain": chain, "source": map.location(*span) });
                if effect == Effects::UNSAFE {
                    entry["obligations"] = serde_json::json!(chain
                        .iter()
                        .filter_map(|name| obligation_of(name).map(|obligation| serde_json::json!({ "function": name, "obligation": obligation })))
                        .collect::<Vec<_>>());
                }
                entry
            })
            .collect();
        let mut report = serde_json::json!({ "schema": 1, "item": request.item, "effect": word, "chains": chains });
        if request.deterministic {
            report["deterministic"] = serde_json::json!(shown.is_empty());
            report["asserted"] = serde_json::json!(asserted);
        }
        println!("{report}");
        return Ok(());
    }
    if request.deterministic {
        if shown.is_empty() && asserted.is_empty() {
            println!("`{}` satisfies [DET-1]", request.item);
        } else if shown.is_empty() {
            println!("`{}` satisfies [DET-1] on asserted foreign facts, not verified:", request.item);
            for chain in &asserted {
                println!("  {} (`@ffi(deterministic)` asserted)", chain.join(" -> "));
            }
        } else {
            println!("`{}` does not satisfy [DET-1]: it reaches {} `Nondet` source(s)", request.item, shown.len());
            for (span, chain) in shown {
                println!("  {}  ({})", chain.join(" -> "), map.location(*span));
            }
        }
        return Ok(());
    }
    let Some((span, chain)) = shown.first() else {
        println!("`{}` does not reach `{word}`", request.item);
        return Ok(());
    };
    println!("`{}` reaches `{word}`:", request.item);
    println!("  {}  ({})", chain.join(" -> "), map.location(*span));
    if effect == Effects::UNSAFE {
        for name in chain {
            match obligation_of(name) {
                Some(Some(obligation)) => println!("  `{name}` requires: {obligation}"),
                Some(None) => println!("  `{name}` states no obligation"),
                None => {}
            }
        }
    }
    Ok(())
}

/// `[UNS-7]` — every function's declaration and the obligation it states (`@safety`), standard ones
/// too.
fn function_obligations(modules: &[ember_typeck::LoadedModule]) -> Vec<(Span, Option<String>)> {
    let mut obligations: Vec<(Span, Option<String>)> = Vec::new();
    for loaded in modules {
        for item in &loaded.module.items {
            match &item.kind {
                ember_ast::ItemKind::Fn(_) => obligations.push((item.span, safety_obligation(&item.attrs))),
                ember_ast::ItemKind::Struct(ember_ast::StructDecl { members, .. })
                | ember_ast::ItemKind::Class(ember_ast::ClassDecl { members, .. })
                | ember_ast::ItemKind::Enum(ember_ast::EnumDecl { members, .. })
                | ember_ast::ItemKind::Extend(ember_ast::ExtendDecl { members, .. }) => {
                    for member in members {
                        if matches!(member.kind, ember_ast::MemberKind::Fn(_)) {
                            obligations.push((member.span, safety_obligation(&member.attrs)));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    obligations
}

/// `[CLI-13]` — each foreign function `request.item` can reach, with the chain to it and the
/// contract its declaration states (each fact with its grade, `[TCB-1]`).
fn print_foreign_calls(
    input: &Path,
    bodies: &[ember_mir::Body],
    types: &TypeTable,
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
    request: &ForeignCalls,
) -> Result<(), String> {
    let reach = ember_analysis::effects::foreign_reach(bodies, types, &request.item)
        .ok_or_else(|| format!("no function named `{}` in {}", request.item, input.display()))?;
    let module_name = |loaded: &ember_typeck::LoadedModule| {
        if loaded.path.is_empty() {
            input.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default()
        } else {
            loaded.path.join(".")
        }
    };
    let declarations = foreign::declarations(modules, map, &module_name);
    let declaration = |index: usize| declarations.iter().find(|decl| decl.qualified == bodies[index].name);
    if request.json {
        let reached: Vec<serde_json::Value> = reach.reached.iter().map(|(index, chain)| {
            serde_json::json!({
                "function": bodies[*index].name,
                "symbol": ember_branding::export_alias_symbol(&bodies[*index].symbol).unwrap_or_else(|| bodies[*index].symbol.clone()),
                "chain": chain,
                "declaration": declaration(*index).map(|decl| foreign::json(decl, map)),
            })
        }).collect();
        println!("{}", serde_json::json!({
            "schema": 1,
            "item": request.item,
            "foreign": reached,
            "unknown_callees": reach.unknown_callees,
        }));
        return Ok(());
    }
    println!("Foreign functions `{}` can reach ({}):", request.item, reach.reached.len());
    for (index, chain) in &reach.reached {
        println!("  {}", chain.join(" -> "));
        match declaration(*index) {
            Some(decl) => foreign::print(decl, map, "    ", None),
            None => println!("    `{}` (C symbol {})", bodies[*index].name, bodies[*index].symbol),
        }
    }
    for function in &reach.unknown_callees {
        println!("  and what `{function}` calls through a function or `dyn` value, which nothing here names");
    }
    Ok(())
}

/// `[UNS-8]` — one `unsafe:` block or `unsafe fn` of the program: where, which function, its
/// safety note (`[LEX-23]`), and what its body may reach only through `unsafe`.
struct UnsafeEntry {
    site: Span,
    extent: Span,
    block: bool,
    function: String,
    module: String,
    note: Option<(Option<&'static str>, String)>,
    calls: Vec<(String, Option<String>)>,
}

/// The functions of a module's items, with the spans of their declarations.
fn module_functions(module: &ember_ast::Module) -> Vec<(Span, &ember_ast::FnDecl)> {
    let mut out = Vec::new();
    for item in &module.items {
        let members = match &item.kind {
            ember_ast::ItemKind::Fn(decl) => {
                out.push((item.span, decl));
                continue;
            }
            ember_ast::ItemKind::Struct(decl) => &decl.members,
            ember_ast::ItemKind::Class(decl) => &decl.members,
            ember_ast::ItemKind::Enum(decl) => &decl.members,
            ember_ast::ItemKind::Interface(decl) => &decl.members,
            ember_ast::ItemKind::Extend(decl) => &decl.members,
            _ => continue,
        };
        for member in members {
            if let ember_ast::MemberKind::Fn(decl) = &member.kind {
                out.push((member.span, decl));
            }
        }
    }
    out
}

/// Each `unsafe:` block in `block`, with its statement's span and its body's.
fn unsafe_blocks(block: &ember_ast::Block, out: &mut Vec<(Span, Span)>) {
    use ember_ast::StmtKind;
    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Unsafe(body) => {
                out.push((stmt.span, body.span));
                unsafe_blocks(body, out);
            }
            StmtKind::If(branch) => unsafe_if_blocks(branch, out),
            StmtKind::While { body, else_block, .. } | StmtKind::For { body, else_block, .. } => {
                unsafe_blocks(body, out);
                if let Some(block) = else_block {
                    unsafe_blocks(block, out);
                }
            }
            StmtKind::Match { arms, .. } => {
                for arm in arms {
                    if let ember_ast::MatchArmBody::Block(body) = &arm.body {
                        unsafe_blocks(body, out);
                    }
                }
            }
            StmtKind::With { body, .. } | StmtKind::Defer(body) | StmtKind::Comptime(body) => unsafe_blocks(body, out),
            StmtKind::LocalFn(decl) => {
                if let Some(body) = &decl.body {
                    unsafe_blocks(body, out);
                }
            }
            _ => {}
        }
    }
}

fn unsafe_if_blocks(branch: &ember_ast::IfStmt, out: &mut Vec<(Span, Span)>) {
    unsafe_blocks(&branch.then_block, out);
    match branch.else_block.as_deref() {
        Some(ember_ast::ElseBranch::Block(block)) => unsafe_blocks(block, out),
        Some(ember_ast::ElseBranch::If(next)) => unsafe_if_blocks(next, out),
        None => {}
    }
}

/// `@safety("…")`, the obligation `[UNS-7]` has an unsafe function state.
fn safety_obligation(attrs: &[ember_ast::Attribute]) -> Option<String> {
    attrs.iter().filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("safety")).find_map(|attr| match attr.args.first() {
        Some(ember_ast::AttrArg::Expr(ember_ast::Expr { kind: ember_ast::ExprKind::Lit(ember_ast::Literal::Str(text)), .. })) => Some(text.clone()),
        _ => None,
    })
}

/// `[UNS-8]`, `[TCB-4]`'s unsafe section — every `unsafe:` block and `unsafe fn` of the program, by
/// module, with its safety note and category, and for each the unsafe functions its body calls
/// with the obligation each states (`@safety`, `[UNS-7]`).
fn print_tcb_report(
    input: &Path,
    bodies: &[ember_mir::Body],
    types: &TypeTable,
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
    request: &TcbReport,
) -> Result<(), String> {
    let module_name = |loaded: &ember_typeck::LoadedModule| {
        if loaded.path.is_empty() {
            input.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default()
        } else {
            loaded.path.join(".")
        }
    };
    let contains = |outer: Span, inner: Span| outer.file == inner.file && outer.start <= inner.start && inner.start < outer.end.max(outer.start + 1);
    // Every function's declaration and its stated obligation, standard ones too: a program's
    // `unsafe` calls them.
    let obligations = function_obligations(modules);
    let mut entries: Vec<UnsafeEntry> = Vec::new();
    for loaded in modules {
        if loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE) || loaded.module.unsafe_sites.is_empty() {
            continue;
        }
        let name = module_name(loaded);
        if request.module.as_ref().is_some_and(|wanted| *wanted != name) {
            continue;
        }
        let file = loaded.module.span.file;
        let text = &map.file(file).text;
        let comments = ember_lexer::lex(file, text, &mut Sink::new()).comments;
        let line = |span: Span| map.file(span.file).line_col(span.start).line;
        let functions = module_functions(&loaded.module);
        let mut blocks = Vec::new();
        for (_, decl) in &functions {
            if let Some(body) = &decl.body {
                unsafe_blocks(body, &mut blocks);
            }
        }
        for &site in &loaded.module.unsafe_sites {
            let at = line(site);
            let note = comments
                .iter()
                .filter(|comment| comment.safety.is_some())
                .find(|comment| (line(comment.span) == at && !comment.own_line) || (at > 1 && line(comment.span) == at - 1 && comment.own_line))
                .map(|comment| {
                    let written = &text[comment.span.start as usize..comment.span.end as usize];
                    let reason = written.split_once(':').map_or(written, |(_, reason)| reason).trim().to_string();
                    (comment.safety.as_ref().and_then(|safety| safety.category), reason)
                });
            let owner = functions.iter().filter(|(span, _)| contains(*span, site)).min_by_key(|(span, _)| span.end - span.start);
            let function = owner.map_or_else(String::new, |(_, decl)| decl.name.name.to_string());
            let entry = match blocks.iter().find(|(stmt, _)| stmt.start == site.start && stmt.file == site.file) {
                Some(&(_, body)) => UnsafeEntry { site, extent: body, block: true, function, module: name.clone(), note, calls: Vec::new() },
                None => {
                    let extent = owner.and_then(|(_, decl)| decl.body.as_ref()).map_or(site, |body| body.span);
                    UnsafeEntry { site, extent, block: false, function, module: name.clone(), note, calls: Vec::new() }
                }
            };
            entries.push(entry);
        }
    }
    // The calls into unsafe functions, each placed in the innermost `unsafe` it is written in.
    let by_symbol: std::collections::HashMap<&str, &ember_mir::Body> = bodies.iter().map(|body| (body.symbol.as_str(), body)).collect();
    for body in bodies {
        for block in &body.blocks {
            let call = match &block.terminator {
                ember_mir::Terminator::Call { func: ember_mir::FuncRef::Direct { symbol, .. }, .. } => {
                    let Some(callee) = by_symbol.get(symbol.as_str()) else { continue };
                    if !callee.is_unsafe {
                        continue;
                    }
                    let obligation = obligations
                        .iter()
                        .filter(|(span, _)| contains(*span, callee.span))
                        .min_by_key(|(span, _)| span.end - span.start)
                        .and_then(|(_, obligation)| obligation.clone());
                    (callee.name.clone(), obligation)
                }
                // ODR-115 — through an `unsafe fn(…)` value: which function, and so its
                // obligation, is not known at the call.
                ember_mir::Terminator::Call { func: ember_mir::FuncRef::Indirect { ty, .. }, .. }
                    if matches!(types.kind(*ty), ember_types::TyKind::Fn { is_unsafe: true, .. }) =>
                {
                    (format!("a value of type {}", types.display(*ty)), Some("unknown at the call: the value may be any `unsafe fn` of its type".to_string()))
                }
                _ => continue,
            };
            let at = block.terminator_span;
            let Some(entry) = entries.iter_mut().filter(|entry| contains(entry.extent, at)).min_by_key(|entry| entry.extent.end - entry.extent.start) else {
                continue;
            };
            if !entry.calls.contains(&call) {
                entry.calls.push(call);
            }
        }
    }
    entries.sort_by_key(|entry| (entry.module.clone(), entry.site.file, entry.site.start));
    // `[FFI-10]`, `[TCB-1]`, `[TCB-3]` — every fact the program's foreign declarations assert.
    let mut foreign: Vec<foreign::ForeignDecl> = foreign::declarations(modules, map, &module_name)
        .into_iter()
        .filter(|decl| request.module.as_ref().is_none_or(|wanted| *wanted == decl.module))
        .collect();
    foreign.sort_by(|a, b| a.module.cmp(&b.module));
    // `[FFI-37c]`, `[TCB-5]`, `[TCB-6]` — what an instrumented test run recorded of the effect facts.
    let evidence = foreign::load_evidence(input);
    if request.json {
        let unsafe_json: Vec<serde_json::Value> = entries
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "module": entry.module,
                    "source": map.location(entry.site),
                    "kind": if entry.block { "block" } else { "fn" },
                    "function": entry.function,
                    "note": entry.note.as_ref().map(|(_, reason)| reason.clone()),
                    "category": entry.note.as_ref().and_then(|(category, _)| *category),
                    "calls": entry.calls.iter().map(|(name, obligation)| serde_json::json!({ "function": name, "obligation": obligation })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let foreign_json: Vec<serde_json::Value> = foreign.iter().map(|decl| {
            let mut value = foreign::json(decl, map);
            if let Some(lines) = evidence.as_ref().and_then(|evidence| foreign::evidence_lines(decl, evidence, map)) {
                value["evidence"] = serde_json::json!(lines);
            }
            value
        }).collect();
        let categories: serde_json::Map<String, serde_json::Value> = trusted_base_categories(modules)
            .into_iter()
            .map(|(name, text)| (name.to_string(), serde_json::json!(text)))
            .collect();
        let mut report = serde_json::json!({
            "schema": 1,
            "guaranteed": LANGUAGE_GUARANTEES,
            "categories": categories,
            "unsafe": unsafe_json,
            "foreign": foreign_json,
        });
        if request.audit {
            report["effects"] = serde_json::json!(effect_totals(bodies, types, modules)
                .into_iter()
                .map(|(word, count)| (word.to_string(), serde_json::json!(count)))
                .collect::<serde_json::Map<_, _>>());
        }
        println!("{report}");
        return Ok(());
    }
    // `[CLI-11]` — the summary: the unsafe code, the effects, the foreign boundary.
    if request.audit {
        let blocks = entries.iter().filter(|entry| entry.block).count();
        let noted = entries.iter().filter(|entry| entry.note.is_some()).count();
        println!("Audit: {}", input.display());
        println!("  unsafe: {blocks} block(s) and {} unsafe fn(s), {noted} with a safety note", entries.len() - blocks);
        let totals: Vec<String> = effect_totals(bodies, types, modules)
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(word, count)| format!("{word} {count}"))
            .collect();
        println!("  effects (functions carrying each): {}", if totals.is_empty() { "none".to_string() } else { totals.join(", ") });
        println!("  foreign boundary: {} declaration(s), {} safe to call", foreign.len(), foreign.iter().filter(|decl| decl.safe).count());
        println!();
    }
    // `[TCB-2]` — what the language guarantees, apart from what the program's own `unsafe` and
    // the foreign code it declares supply; each of those with its source and grade (`[TCB-3]`).
    println!("Trusted base: {}", input.display());
    println!("Guaranteed by the language, checked by the compiler: {LANGUAGE_GUARANTEES}");
    // `[TCB-4]` — the rest of the base, by category.
    for (name, text) in trusted_base_categories(modules) {
        println!("{}{}: {text}", name[..1].to_uppercase(), &name[1..]);
    }
    println!("Supplied from outside the language, each fact with its grade:");
    println!("Unsafe ({}):", entries.len());
    let mut module = None;
    for entry in &entries {
        if module != Some(&entry.module) {
            println!("  module {}", entry.module);
            module = Some(&entry.module);
        }
        let what = if entry.block { format!("block in `{}`", entry.function) } else { format!("unsafe fn `{}`", entry.function) };
        let note = match &entry.note {
            Some((Some(category), reason)) => format!("SAFETY({category}): {reason}"),
            Some((None, reason)) => format!("SAFETY: {reason}"),
            None => "no safety note (W3012)".to_string(),
        };
        println!("    {}  {what}  {note}  [asserted]", map.location(entry.site));
        for (name, obligation) in &entry.calls {
            match obligation {
                Some(obligation) => println!("      calls `{name}`: {obligation}"),
                None => println!("      calls `{name}`: states no obligation"),
            }
        }
    }
    println!("C FFI ({} declarations):", foreign.len());
    let mut module = None;
    for decl in &foreign {
        if module != Some(&decl.module) {
            println!("  module {}", decl.module);
            module = Some(&decl.module);
        }
        foreign::print(decl, map, "    ", evidence.as_ref());
    }
    Ok(())
}

/// `[CLI-11]` — for each effect, how many of the program's own functions carry it.
fn effect_totals(
    bodies: &[ember_mir::Body],
    types: &TypeTable,
    modules: &[ember_typeck::LoadedModule],
) -> Vec<(&'static str, usize)> {
    let standard_files: std::collections::BTreeSet<ember_span::FileId> = modules
        .iter()
        .filter(|loaded| loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE))
        .map(|loaded| loaded.module.span.file)
        .collect();
    let std_prefix = ember_branding::mangled(&format!("{}.", ember_branding::STD_PACKAGE));
    let effects = ember_analysis::effect_sets(bodies, types);
    ember_hir::Effects::NAMED
        .iter()
        .map(|&(effect, word)| {
            let count = bodies
                .iter()
                .zip(&effects)
                .filter(|(body, carried)| {
                    !body.symbol.starts_with(&std_prefix) && !standard_files.contains(&body.span.file) && carried.contains(effect)
                })
                .count();
            (word, count)
        })
        .collect()
}

/// `[TCB-2]` — what the language guarantees of a program's safe code (`[TIER-1]`'s Safe tier).
const LANGUAGE_GUARANTEES: &str =
    "memory, null, bounds and lifetime safety, data-race freedom and deterministic destruction of the safe code";

/// `[COST-4]`, `[PHIL-15]` — the implicit costs of the program's own functions that are not checks:
/// allocations, calls through a value, a vtable or an interface, foreign calls, the count operations
/// left in loops, and which `contains` each `x in c` selects (`[STD-8a]`). Each with its row of the
/// cost table (`[COST-3]`) and its class (`[COST-2]`).
fn safety_costs_json(
    bodies: &[ember_mir::Body],
    types: &TypeTable,
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
    contains_uses: &[ember_typeck::ContainsUse],
) -> String {
    let standard_files: std::collections::BTreeSet<ember_span::FileId> = modules
        .iter()
        .filter(|loaded| loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE))
        .map(|loaded| loaded.module.span.file)
        .collect();
    let std_prefix = ember_branding::mangled(&format!("{}.", ember_branding::STD_PACKAGE));
    let standard: Vec<bool> = bodies
        .iter()
        .map(|body| body.symbol.starts_with(&std_prefix) || standard_files.contains(&body.span.file))
        .collect();
    let by_symbol: std::collections::HashMap<&str, &ember_mir::Body> = bodies.iter().map(|body| (body.symbol.as_str(), body)).collect();
    // `[FFI-24c]` — the modules `import cpp` wrote: their functions are the program's C++ calls, and
    // their bodies (the wrappers of the thunks) are the import's, not the program's own.
    let cpp_files: std::collections::BTreeSet<ember_span::FileId> = modules
        .iter()
        .filter(|loaded| loaded.foreign.as_ref().is_some_and(|foreign| foreign.cpp.is_some()))
        .map(|loaded| loaded.module.span.file)
        .collect();
    let mut costs: Vec<serde_json::Value> = Vec::new();
    let mut add = |row: &str, class: &str, span: ember_span::Span, function: &str, detail: String| {
        costs.push(serde_json::json!({
            "row": row, "class": class, "source": map.location(span), "function": function, "detail": detail,
        }));
    };
    for site in ember_analysis::allocation_sites(bodies, types, &standard, &|file| standard_files.contains(&file)) {
        add("Allocation", "guaranteed present", site.span, &site.function, site.operations.join(", "));
    }
    for (index, body) in bodies.iter().enumerate() {
        if standard[index] || cpp_files.contains(&body.span.file) {
            continue;
        }
        for block in &body.blocks {
            let ember_mir::Terminator::Call { func, .. } = &block.terminator else { continue };
            let span = block.terminator_span;
            // `[FFI-24c]` — a C++ call, with its exception mode.
            if let ember_mir::FuncRef::Direct { symbol, .. } = func
                && let Some(callee) = by_symbol.get(symbol.as_str())
                && let Some(mode) = cpp_call_mode(callee, &by_symbol, &cpp_files)
            {
                add("FFI call", "guaranteed present", span, &body.name, format!("`{}`, a C++ call through its thunk: {mode} [FFI-24c]", callee.name));
                continue;
            }
            match func {
                ember_mir::FuncRef::Interface { .. } => add("Interface call through `dyn`", "guaranteed present", span, &body.name, "one indirect call".to_string()),
                ember_mir::FuncRef::Virtual { .. } => add("Virtual method call", "conditionally elided", span, &body.name, "one indirect call through the class's method table".to_string()),
                ember_mir::FuncRef::Indirect { .. } => add("Owned callable value", "guaranteed present", span, &body.name, "one indirect call".to_string()),
                ember_mir::FuncRef::Direct { symbol, .. } => {
                    if let Some(callee) = by_symbol.get(symbol.as_str()) && callee.contracts.foreign.is_some() {
                        // `[FFI-29c]` — a `static inline` function is reached through the shim's wrapper.
                        let exact = ember_branding::export_alias_symbol(symbol).unwrap_or_else(|| symbol.clone());
                        let detail = if exact.starts_with(&format!("{}_cshim_", ember_branding::SYMBOL_PREFIX)) {
                            format!("`{}`, wrapped (static inline): a direct call to the import's shim, which calls it \
                                     (one call more, unless the shim takes part in link-time optimisation)", callee.name)
                        } else {
                            format!("`{}`, one direct call", callee.name)
                        };
                        add("FFI call", "guaranteed present", span, &body.name, detail);
                    }
                    // `[FFI-17f]` — an operation of a C++ bridge type is a call of its C++ thunk.
                    if let Some(callee) = by_symbol.get(symbol.as_str())
                        && symbol.starts_with(&ember_branding::mangled("std.ffi."))
                        && let Some((_, receiver)) = callee.args().next()
                        && let Some(bridge) = bridge_type(types, receiver.ty)
                    {
                        let method = callee.name.rsplit('.').next().unwrap_or(&callee.name);
                        if matches!(method, "as_ptr" | "from_thunks") {
                            continue;
                        }
                        add("FFI call", "guaranteed present", span, &body.name, format!("`{bridge}.{method}`, through its C++ thunk: a call, never inlined [FFI-17f]"));
                    }
                }
                _ => {}
            }
        }
        // `[FFI-17f]` — a bridge value's end is a call of its C++ destructor's thunk.
        for block in &body.blocks {
            for stmt in &block.stmts {
                if let ember_mir::StmtKind::Drop { place, .. } = &stmt.kind
                    && place.projection.is_empty()
                    && let Some(bridge) = bridge_type(types, body.local(place.local).ty)
                {
                    add("FFI call", "guaranteed present", stmt.span, &body.name, format!("`{bridge}` dropped, through its C++ destructor's thunk: a call, never inlined [FFI-17f]"));
                }
            }
        }
    }
    for operation in ember_analysis::count_report::surviving_count_operations(bodies, types) {
        let what = if operation.retain { "a retain" } else { "a release" };
        add("Retain / release, surviving", "guaranteed present", operation.span, &operation.function, format!("{what} in a loop: {}", operation.reason));
    }
    for use_ in contains_uses {
        if standard_files.contains(&use_.span.file) {
            continue;
        }
        add("`contains`", "guaranteed present", use_.span, "", format!("{}: {}", use_.selects, use_.cost));
    }
    serde_json::Value::Array(costs).to_string()
}

/// `[FFI-24c]` — the exception mode of a call of `callee`, when it is a function `import cpp` wrote:
/// `noexcept (derived)` for a thunk called directly, or a wrapper whose thunk does not catch;
/// `noexcept (asserted)` for one an overlay asserts `@ffi(noexcept)`, whose thunk catches and whose
/// wrapper panics on a throw (`[FFI-24b]`); `catching` for one that returns the throw as a
/// `CppError` (`[FFI-24]`). `None` for any other function, and for a helper of the import's that
/// calls no thunk.
fn cpp_call_mode(
    callee: &ember_mir::Body,
    by_symbol: &std::collections::HashMap<&str, &ember_mir::Body>,
    cpp_files: &std::collections::BTreeSet<ember_span::FileId>,
) -> Option<&'static str> {
    if !cpp_files.contains(&callee.span.file) {
        return None;
    }
    if callee.contracts.foreign.is_some() {
        return Some("noexcept (derived)");
    }
    let prefix = ember_branding::SYMBOL_PREFIX;
    let called: Vec<&str> = callee
        .blocks
        .iter()
        .filter_map(|block| match &block.terminator {
            ember_mir::Terminator::Call { func: ember_mir::FuncRef::Direct { symbol, .. }, .. } => Some(symbol.as_str()),
            _ => None,
        })
        .collect();
    let thunk = called.iter().any(|symbol| by_symbol.get(symbol).is_some_and(|body| body.contracts.foreign.is_some() && cpp_files.contains(&body.span.file)));
    if !thunk {
        return None;
    }
    Some(if called.iter().any(|symbol| symbol.contains(&format!("{prefix}_cpp_value"))) {
        "noexcept (asserted)"
    } else if called.iter().any(|symbol| symbol.contains(&format!("{prefix}_cpp_result"))) {
        "catching"
    } else {
        "noexcept (derived)"
    })
}

/// `[FFI-17e]` — the name of a C++ bridge type (`std.ffi`'s `CppVector`, `CppString`, `CppShared`),
/// for `ty` or a reference to one.
fn bridge_type(types: &TypeTable, ty: ember_types::Ty) -> Option<&'static str> {
    let ty = match types.kind(ty) {
        ember_types::TyKind::Ref { inner, .. } => *inner,
        _ => ty,
    };
    let ember_types::TyKind::Struct(id) = types.kind(ty) else { return None };
    let def = types.struct_def(*id);
    let name = def.origin.as_ref().map_or(def.name, |(name, _)| *name);
    ["CppVector", "CppString", "CppShared"].into_iter().find(|bridge| name.as_str() == format!("std.ffi.{bridge}"))
}

/// `[EFF-16]` — each function of the program with the two facts that together say it has no panic
/// the effect model knows of: whether it is `@nopanic(explicit)`
/// (written, or through `@realtime`), and the `RuntimeCheck` kinds it reaches, its callees' included.
fn safety_functions_json(
    bodies: &[ember_mir::Body],
    types: &TypeTable,
    modules: &[ember_typeck::LoadedModule],
    reloadable: Option<&std::collections::BTreeSet<String>>,
) -> String {
    use ember_hir::Effects;
    let standard_files: std::collections::BTreeSet<ember_span::FileId> = modules
        .iter()
        .filter(|loaded| loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE))
        .map(|loaded| loaded.module.span.file)
        .collect();
    let std_prefix = ember_branding::mangled(&format!("{}.", ember_branding::STD_PACKAGE));
    let effects = ember_analysis::effect_sets(bodies, types);
    let checks = [
        (Effects::CHECK_ARITHMETIC, "Arithmetic"),
        (Effects::CHECK_BOUNDS, "Bounds"),
        (Effects::CHECK_ALIASING, "Aliasing"),
        (Effects::CHECK_STALE, "Stale"),
    ];
    let functions: Vec<serde_json::Value> = bodies
        .iter()
        .zip(&effects)
        .filter(|(body, _)| !body.symbol.starts_with(&std_prefix) && !standard_files.contains(&body.span.file))
        .map(|(body, effects)| {
            let contracts = body.contracts.forbidden | body.contracts.realtime;
            let kinds: Vec<&str> = checks.iter().filter(|(effect, _)| effects.contains(*effect)).map(|(_, name)| *name).collect();
            let mut entry = serde_json::json!({
                "function": body.name,
                "nopanic_explicit": contracts.contains(Effects::PANIC),
                "runtime_checks": kinds,
            });
            // `[HR-28]` — in a reloadable build, whether a call reaches the function through its
            // thunk or directly.
            if let Some(reloadable) = reloadable {
                entry["call"] = serde_json::json!(if reloadable.contains(&body.symbol) { "thunk" } else { "direct" });
            }
            entry
        })
        .collect();
    serde_json::to_string(&functions).unwrap_or_else(|_| "[]".to_string())
}

/// `[PHIL-2]`, `[CLO-10]` — print each place a function of the program
/// allocates, and what allocates there.
fn print_alloc_report(
    input: &Path,
    bodies: &[ember_mir::Body],
    types: &TypeTable,
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
    request: &AllocReport,
) -> Result<(), String> {
    let standard_files: std::collections::BTreeSet<ember_span::FileId> = modules
        .iter()
        .filter(|loaded| loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE))
        .map(|loaded| loaded.module.span.file)
        .collect();
    let std_prefix = ember_branding::mangled(&format!("{}.", ember_branding::STD_PACKAGE));
    let standard: Vec<bool> = bodies
        .iter()
        .map(|body| body.symbol.starts_with(&std_prefix) || standard_files.contains(&body.span.file))
        .collect();
    let sites: Vec<ember_analysis::AllocationSite> =
        ember_analysis::allocation_sites(bodies, types, &standard, &|file| standard_files.contains(&file))
            .into_iter()
            .filter(|site| request.function.as_deref().is_none_or(|name| site.function == name))
            .collect();
    // `[SEL-1]` (ODR-108) — and the mechanism each declaration uses; with `--function`, its
    // parameters and locals.
    let declarations: Vec<ember_analysis::Declaration> =
        ember_analysis::declarations(bodies, types, &standard, &|file| standard_files.contains(&file))
            .into_iter()
            .filter(|declaration| {
                request.function.as_deref().is_none_or(|name| declaration.owner == name && declaration.kind != "field")
            })
            .collect();
    if request.json {
        let sites: Vec<serde_json::Value> = sites
            .iter()
            .map(|site| {
                serde_json::json!({
                    "function": site.function,
                    "source": map.location(site.span),
                    "allocates": site.operations,
                })
            })
            .collect();
        let declarations: Vec<serde_json::Value> = declarations
            .iter()
            .map(|declaration| {
                serde_json::json!({
                    "kind": declaration.kind,
                    "owner": declaration.owner,
                    "name": declaration.name,
                    "type": declaration.ty,
                    "mechanism": declaration.mechanism,
                    "source": map.location(declaration.span),
                })
            })
            .collect();
        println!("{}", serde_json::json!({ "schema": 1, "sites": sites, "declarations": declarations }));
        return Ok(());
    }
    println!("Allocation sites: {}", input.display());
    if let Some(function) = &request.function {
        println!("Function: {function}");
    }
    if sites.is_empty() {
        println!("  none");
    }
    for site in &sites {
        println!("  {} {}: {}", site.function, map.location(site.span), site.operations.join(", "));
    }
    println!("Storage mechanisms:");
    if declarations.is_empty() {
        println!("  none");
    }
    for declaration in &declarations {
        let name = match declaration.kind {
            "field" => format!("{}.{}", declaration.owner, declaration.name),
            "type" => format!("{} {}", declaration.ty, declaration.name),
            _ => format!("{} {} `{}`", declaration.owner, declaration.kind, declaration.name),
        };
        let ty = if declaration.kind == "type" { String::new() } else { format!(": {}", declaration.ty) };
        println!("  {}: {name}{ty} -> {}", map.location(declaration.span), declaration.mechanism);
    }
    Ok(())
}

/// `[RC-6]` — print the count operations a build left inside loops.
fn inspect_counts(path: &Path, function: Option<&str>, json: bool) -> Result<ExitCode, String> {
    let resolved = resolve_side_table(path, "counts");
    let text = std::fs::read_to_string(&resolved)
        .map_err(|error| format!("could not read count side table `{}`: {error}", resolved.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| format!("invalid count side table `{}`: {error}", resolved.display()))?;
    if value.get("schema").and_then(serde_json::Value::as_u64) != Some(1) {
        return Err("unsupported or missing count side-table schema".to_string());
    }
    let operations: Vec<&serde_json::Value> = value
        .get("operations")
        .and_then(serde_json::Value::as_array)
        .ok_or("count side table is missing its operations array")?
        .iter()
        .filter(|operation| function.is_none_or(|name| operation.get("function").and_then(serde_json::Value::as_str) == Some(name)))
        .collect();
    if json {
        println!("{}", serde_json::json!({ "schema": 1, "operations": operations }));
        return Ok(ExitCode::SUCCESS);
    }
    let field = |operation: &serde_json::Value, name: &str| {
        operation.get(name).and_then(serde_json::Value::as_str).unwrap_or("<unknown>").to_string()
    };
    println!("Count operations inside loops: {}", resolved.display());
    if let Some(function) = function {
        println!("Function: {function}");
    }
    if operations.is_empty() {
        println!("  none");
        return Ok(ExitCode::SUCCESS);
    }
    let retains = operations.iter().filter(|operation| field(operation, "kind") == "retain").count();
    println!("  retain {retains}");
    println!("  release {}", operations.len() - retains);
    for operation in operations {
        println!(
            "  {} {}: {} in the loop at {}: {}",
            field(operation, "function"),
            field(operation, "source"),
            field(operation, "kind"),
            field(operation, "loop"),
            field(operation, "reason"),
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn resolve_side_table(path: &Path, kind: &str) -> PathBuf {
    if path.is_file() {
        return path.to_path_buf();
    }
    for directory in ["target/debug/inspect", "target/development/inspect", "target/release/inspect"] {
        let candidate = PathBuf::from(directory).join(format!("{}.{kind}.json", path.display()));
        if candidate.is_file() {
            return candidate;
        }
    }
    path.to_path_buf()
}

fn resolve_safety_path(path: &Path) -> PathBuf {
    if path.is_file() {
        return path.to_path_buf();
    }
    let candidates = [
        PathBuf::from("target/debug/inspect"),
        PathBuf::from("target/development/inspect"),
        PathBuf::from("target/release/inspect"),
    ];
    for directory in candidates {
        let candidate = directory.join(format!("{}.safety.json", path.display()));
        if candidate.is_file() {
            return candidate;
        }
    }
    path.to_path_buf()
}

fn validate_safety_check(check: &serde_json::Value) -> Result<(), String> {
    let object = check
        .as_object()
        .ok_or("safety side-table check is not an object")?;
    for field in ["kind", "source", "function", "mechanism", "reason", "status"] {
        if object.get(field).and_then(serde_json::Value::as_str).is_none() {
            return Err(format!("safety side-table check is missing string field `{field}`"));
        }
    }
    match object.get("status").and_then(serde_json::Value::as_str) {
        Some("emitted" | "elided") => {}
        Some(status) => return Err(format!("unknown safety side-table status `{status}`")),
        None => unreachable!("status was checked above"),
    }
    let Some(classification) = object
        .get("classification")
        .and_then(serde_json::Value::as_str)
    else {
        // `classification` was added to schema 1 rather than forcing every
        // previously-emitted side table through a migration. Its legacy
        // meaning is inferred at rendering time from `status`.
        return Ok(());
    };
    match classification {
        "DYNAMIC_PER_ACCESS" | "STATIC_ELIDED" => Ok(()),
        "DYNAMIC_HOISTED_LOOP" => {
            for field in ["proof", "loop", "check_site", "protected_interval"] {
                if object.get(field).and_then(serde_json::Value::as_str).is_none() {
                    return Err(format!(
                        "hoisted safety side-table check is missing string field `{field}`"
                    ));
                }
            }
            if object.get("check_site").and_then(serde_json::Value::as_str) != Some("preheader") {
                return Err("hoisted safety side-table check must use preheader check_site".to_string());
            }
            if object
                .get("protected_interval")
                .and_then(serde_json::Value::as_str)
                != Some("loop")
            {
                return Err("hoisted safety side-table check must protect the loop interval".to_string());
            }
            Ok(())
        }
        other => Err(format!("unknown safety side-table classification `{other}`")),
    }
}

fn print_safety_report(
    path: &Path,
    checks: &[serde_json::Value],
    elided_only: bool,
    function: Option<&str>,
) {
    println!("Safety checks: {}", path.display());
    println!(
        "{}:",
        if elided_only {
            "Elided checks"
        } else {
            "Runtime checks"
        }
    );
    if let Some(function) = function {
        println!("Function: {function}");
    }
    if checks.is_empty() {
        println!("  none");
        return;
    }

    let mut counts = std::collections::BTreeMap::new();
    for check in checks {
        let kind = check
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        *counts.entry(kind).or_insert(0usize) += 1;
    }
    for (kind, count) in counts {
        println!("  {kind} {count}");
    }
    for check in checks {
        let source = check
            .get("source")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<unknown>");
        let function = check
            .get("function")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<unknown>");
        let mechanism = check
            .get("mechanism")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<unknown>");
        let reason = check
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<unknown>");
        let classification = check
            .get("classification")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                if check.get("status").and_then(serde_json::Value::as_str) == Some("elided") {
                    "STATIC_ELIDED"
                } else {
                    "DYNAMIC_PER_ACCESS"
                }
            });
        let loop_name = check
            .get("loop")
            .and_then(serde_json::Value::as_str);
        let proof = check
            .get("proof")
            .and_then(serde_json::Value::as_str);
        let check_site = check
            .get("check_site")
            .and_then(serde_json::Value::as_str);
        let interval = check
            .get("protected_interval")
            .and_then(serde_json::Value::as_str);
        print!(
            "    {source}  {function}  {classification}  {mechanism}  reason: {reason}"
        );
        if let Some(loop_name) = loop_name {
            print!("  loop: {loop_name}");
        }
        if let Some(proof) = proof {
            print!("  proof: {proof}");
        }
        if let Some(check_site) = check_site {
            print!("  check-site: {check_site}");
        }
        if let Some(interval) = interval {
            print!("  interval: {interval}");
        }
        println!();
    }
}

/// `[EFF-16]` — each function's `@nopanic(explicit)` beside its `RuntimeCheck` kinds; both together
/// mean no panic the effect model knows of, though it can still stop on what the model does not
/// cover (`[PHIL-11]`).
fn print_safety_functions(functions: &[serde_json::Value]) {
    if functions.is_empty() {
        return;
    }
    println!("Functions:");
    for entry in functions {
        let name = entry.get("function").and_then(serde_json::Value::as_str).unwrap_or("<unknown>");
        let nopanic = entry.get("nopanic_explicit").and_then(serde_json::Value::as_bool).unwrap_or(false);
        let kinds: Vec<&str> = entry
            .get("runtime_checks")
            .and_then(serde_json::Value::as_array)
            .map(|kinds| kinds.iter().filter_map(serde_json::Value::as_str).collect())
            .unwrap_or_default();
        let checks = if kinds.is_empty() { "none".to_string() } else { kinds.join(", ") };
        let contract = if nopanic { "  @nopanic(explicit)" } else { "" };
        let verdict = if nopanic && kinds.is_empty() { "  (no panic the effect model knows of)" } else { "" };
        // `[HR-28]` — a reloadable build's thunk indirection, beside the checks.
        let call = match entry.get("call").and_then(serde_json::Value::as_str) {
            Some("thunk") => "  Call: through its thunk (one load and one indirect jump)",
            Some(_) => "  Call: direct",
            None => "",
        };
        println!("  {name}{contract}  RuntimeCheck: {checks}{verdict}{call}");
    }
}

fn format_file(input: &Path, check_only: bool, write: bool, json: bool) -> Result<ExitCode, String> {
    let options = Options { json, ..Options::default() };
    let mut map = SourceMap::new();
    let mut sink = Sink::new();
    let file = match map.load(input) {
        Ok(file) => file,
        Err(error @ LoadError::NotUtf8(_)) => {
            sink.emit(load_diagnostic(&error, Span::DUMMY));
            report(&sink, &map, &options);
            return Ok(ExitCode::FAILURE);
        }
        Err(error) => return Err(error.to_string()),
    };
    let source = map.file(file).text.clone();

    let lexed = ember_lexer::lex(file, &source, &mut sink);
    let module = ember_parser::parse(file, &source, lexed.tokens, &mut sink);
    if sink.has_errors() {
        report(&sink, &map, &options);
        return Ok(ExitCode::FAILURE);
    }

    let formatted = ember_fmt::format(&module, &source, &lexed.comments);
    // `[CLI-1]` — whether the file was already in the canonical layout, and
    // the layout itself when it is printed.
    if json {
        if write && formatted != source {
            std::fs::write(input, &formatted).map_err(|e| e.to_string())?;
        }
        let printed = (!check_only && !write).then_some(formatted.as_str());
        println!(
            "{}",
            serde_json::json!({ "schema": 1, "file": input.display().to_string(), "formatted": formatted == source, "text": printed })
        );
        return Ok(if check_only && formatted != source { ExitCode::FAILURE } else { ExitCode::SUCCESS });
    }
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

/// `[CLI-20]` — `ember fmt --migrate`: source written for 0.9.8 rewritten into 0.9.9 where the
/// change is mechanical, then a list of what changed and of what could not be rewritten. Each
/// rewrite is a migration suggestion on the diagnostic the old form meets (the parser's for `::`
/// and `@latebound`, the checker's for integer `/`, `Box[dyn fn]`, a pair loop over a `Map` and
/// `with_views`, the loader's for `std.borrow`); they are applied, and the file checked again,
/// until none is left. `@view` meets no diagnostic on a view type: it is documentation since 0.9.9
/// (`[TYP-34]`) and is removed outright. Only the named file is rewritten.
fn migrate_file(input: &Path, write: bool, json: bool) -> Result<ExitCode, String> {
    const ROUNDS: usize = 8;
    let mut original = SourceMap::new();
    let original_file = original.load(input).map_err(|error| error.to_string())?;
    let mut text = original.file(original_file).text.clone();
    let mut changed: Vec<String> = Vec::new();
    let mut left: Vec<String> = Vec::new();
    // Each round's applied edits, so a place in a later round's text is reported where it is in
    // the file as written.
    let mut rounds: Vec<Vec<(usize, usize, usize)>> = Vec::new();
    let located = |span: Span, rounds: &[Vec<(usize, usize, usize)>]| {
        let mut at = span.start as usize;
        for round in rounds.iter().rev() {
            let mut shift = 0isize;
            for &(start, end, written) in round {
                let new_start = (start as isize + shift) as usize;
                if at < new_start {
                    break;
                }
                if at < new_start + written {
                    at = new_start;
                    break;
                }
                shift += written as isize - (end - start) as isize;
            }
            at = (at as isize - shift) as usize;
        }
        original.location(Span::new(original_file, at as u32, at as u32))
    };
    for _ in 0..ROUNDS {
        let mut map = SourceMap::new();
        let file = map.add(input, text.clone());
        let mut sink = Sink::new();
        let lexed = ember_lexer::lex(file, &text, &mut sink);
        let module = ember_parser::parse(file, &text, lexed.tokens, &mut sink);
        let mut edits: Vec<(Span, String)> = Vec::new();
        // `[TYP-34]` — `@view` on a struct or an enum.
        for item in &module.items {
            for attr in item.attrs.iter().filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("view")) {
                edits.push((attr.span, String::new()));
                changed.push(format!("{}: `@view` removed", located(attr.span, &rounds)));
            }
        }
        if !sink.has_errors() {
            let root_dir = package_source_root(input);
            let lints = manifest_lint_settings(&root_dir, &mut map, &mut sink);
            let modules = load_modules(module, &root_dir, &mut map, &mut sink);
            if !sink.has_errors() {
                let (mut types, common) = TypeTable::new();
                let mark = sink.mark();
                let checked = ember_typeck::check(
                    &modules,
                    &mut types,
                    &common,
                    &mut sink,
                    overflow_policy(Profile::Debug),
                    lints.return_intersection,
                    true,
                    &ember_typeck::ComptimeAnswers::new(),
                    lints.realtime,
                );
                // What a compile-time evaluation would answer is not this command's.
                if !checked.comptime.is_empty() {
                    sink.rollback(mark);
                }
            }
        }
        left.clear();
        for diagnostic in sink.diagnostics() {
            let mut migrations = diagnostic.suggestions.iter().filter(|s| s.migration).peekable();
            if migrations.peek().is_none() {
                if diagnostic.severity == ember_diag::Severity::Error {
                    let at = if diagnostic.primary.span.file == file {
                        located(diagnostic.primary.span, &rounds)
                    } else {
                        map.location(diagnostic.primary.span)
                    };
                    left.push(format!("{at}: {}", diagnostic.message));
                }
                continue;
            }
            for suggestion in migrations {
                if suggestion.edits.iter().all(|(span, _)| span.file == file) {
                    edits.extend(suggestion.edits.iter().map(|(span, with)| (*span, with.text(&map).to_string())));
                    changed.push(format!("{}: {}", located(diagnostic.primary.span, &rounds), suggestion.message));
                }
            }
        }
        if edits.is_empty() {
            break;
        }
        let (migrated, applied) = apply_migration_edits(&text, edits);
        text = migrated;
        rounds.push(applied);
    }
    // In the canonical layout, as `ember fmt` prints, when the result parses.
    let mut map = SourceMap::new();
    let file = map.add(input, text.clone());
    let mut sink = Sink::new();
    let lexed = ember_lexer::lex(file, &text, &mut sink);
    let module = ember_parser::parse(file, &text, lexed.tokens, &mut sink);
    if !sink.has_errors() {
        text = ember_fmt::format(&module, &text, &lexed.comments);
    }
    if json {
        println!(
            "{}",
            serde_json::json!({ "schema": 1, "file": input.display().to_string(), "changed": changed, "not_rewritten": left, "text": (!write).then_some(&text) })
        );
    } else {
        for change in &changed {
            eprintln!("changed: {change}");
        }
        for problem in &left {
            eprintln!("not rewritten: {problem}");
        }
    }
    if write {
        std::fs::write(input, &text).map_err(|e| e.to_string())?;
    } else if !json {
        print!("{text}");
    }
    Ok(if left.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}

/// `[CLI-20]` — `edits` applied to `text`, in order where two share a place; one overlapping an
/// earlier edit is left out. A deletion takes the spaces after it, and its line when nothing else
/// is left on it. Also each applied edit: where it was, and how long its text is.
fn apply_migration_edits(text: &str, mut edits: Vec<(Span, String)>) -> (String, Vec<(usize, usize, usize)>) {
    let bytes = text.as_bytes();
    for (span, with) in &mut edits {
        if !with.is_empty() {
            continue;
        }
        let (mut start, mut end) = (span.start as usize, span.end as usize);
        while end < bytes.len() && matches!(bytes[end], b' ' | b'\t') {
            end += 1;
        }
        let line_start = text[..start].rfind('\n').map_or(0, |at| at + 1);
        let blank_before = text[line_start..start].bytes().all(|b| matches!(b, b' ' | b'\t'));
        if blank_before && (end == bytes.len() || bytes[end] == b'\n' || bytes[end] == b'\r') {
            start = line_start;
            end = text[end..].find('\n').map_or(bytes.len(), |at| end + at + 1);
        }
        *span = Span::new(span.file, start as u32, end as u32);
    }
    edits.sort_by_key(|(span, _)| (span.start, span.end));
    let mut out = String::with_capacity(text.len());
    let mut applied = Vec::with_capacity(edits.len());
    let mut at = 0usize;
    for (span, with) in edits {
        let (start, end) = (span.start as usize, span.end as usize);
        if start < at {
            continue;
        }
        out.push_str(&text[at..start]);
        out.push_str(&with);
        applied.push((start, end, with.len()));
        at = end;
    }
    out.push_str(&text[at..]);
    (out, applied)
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        let value = |index: &mut usize, name: &str| -> Result<String, String> {
            *index += 1;
            args.get(*index)
                .cloned()
                .ok_or_else(|| format!("`{name}` needs a value"))
        };
        match arg {
            "--profile" => {
                options.profile = Profile::from_name(&value(&mut index, arg)?)
                    .ok_or("profile must be debug, development or release")?
            }
            "--emit" => options.emit = Some(value(&mut index, arg)?),
            "--emit-header" => options.emit_header = true,
            "--emit-optimization-report" => options.optimization_report = true,
            "--timings" => options.timings = Some(false),
            "--timings=json" => options.timings = Some(true),
            // `[MONO-2]` — the instances each generic produced and the time they took.
            "--report=instantiations" => options.instantiation_report = true,
            "--report=engine" => options.engine_report = true,
            "--build-id" => options.build_id = true,
            "--require-evidence" => options.require_evidence = true,
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
            "--leak-check" => options.leak_check = true,
            "--no-leak-check" => options.no_leak_check = true,
            "--reload" => options.reload = true,
            "--explain" => options.explain = true,
            "--hot" => options.hot = true,
            // `[MAN-2]` — fail rather than change `ember.lock`.
            "--locked" => options.locked = true,
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
    let body = entries.join(
        "
",
    ) + "
";
    let _ = std::fs::write(&path, body);
    eprintln!(
        "note: {} unclassified borrow error(s) recorded in {} (DIA-7)",
        entries.len(),
        path.display()
    );
}

include!(concat!(env!("OUT_DIR"), "/error_pages.rs"));

fn explain(code: &str, json: bool) -> Result<ExitCode, String> {
    let Some(found) = ember_diag::codes::lookup(code) else {
        return Err(format!("`{code}` is not a diagnostic code"));
    };
    // `[DOC-1]` — the page built into the compiler.
    let page = ERROR_PAGES.iter().find(|(page, _)| *page == found.to_string()).map(|(_, text)| text.to_string());
    if json {
        println!(
            "{}",
            serde_json::json!({
                "schema": 1,
                "code": found.to_string(),
                "title": found.title,
                "rule": found.rule,
                "subsystem": format!("{:?}", found.subsystem),
                "page": page,
            })
        );
        return Ok(ExitCode::SUCCESS);
    }
    println!("{found}: {}", found.title);
    println!();
    println!("Specified by {} ({:?}).", found.rule, found.subsystem);
    // `[DIA-6]` — docs/errors/EXXXX.md exists for every code. The pages are
    // written as each code gains its first test; until then, say so plainly
    // rather than printing an empty page.
    match page {
        Some(text) => {
            println!();
            print!("{text}");
        }
        None => {
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
/// `[MOD-3]` — the module paths an import can name: the standard library's, and the package's
/// under its source root (three directories deep, which a single file's directory may need).
fn known_module_paths(root_dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, prefix: &str, depth: usize, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else { continue };
            let name = if prefix.is_empty() { stem.to_string() } else { format!("{prefix}.{stem}") };
            if path.is_dir() {
                if depth < 3 && !stem.starts_with('.') && stem != "target" {
                    walk(&path, &name, depth + 1, out);
                }
            } else if path.extension().is_some_and(|ext| ext == ember_branding::SOURCE_EXT) {
                out.push(if stem == "mod" { prefix.to_string() } else { name });
            }
        }
    }
    let mut paths = Vec::new();
    if let Some(std_root) = ember_branding::std_root() {
        walk(&std_root, ember_branding::STD_PACKAGE, 0, &mut paths);
    }
    walk(root_dir, "", 0, &mut paths);
    paths.retain(|path| !path.is_empty() && path != "std.lib");
    paths
}

/// The standard modules the specification names (Part XV's table), whether or not this compiler
/// has written them yet.
const SPECIFIED_STD_MODULES: &[&str] = &[
    "arena", "cell", "collections", "core", "ecs", "ffi", "fmt", "gpu", "io", "math", "mem", "process", "random",
    "ser", "simd", "string", "testing", "thread", "time",
];

/// `[MOD-1]` — the module an import inside package `package` names, with the import rewritten to
/// the module's path in the build when that differs from what is written (a dependency's second
/// major, `[VER-5]`; a package's own module inside a dependency), binding the same name. `None`
/// when no package has it: the caller tries the standard library.
fn resolve_package_import(
    packages: &packages::Packages,
    package: usize,
    import: &mut ember_ast::Import,
) -> Option<(usize, Vec<String>, PathBuf)> {
    let (segments, alias) = match &mut import.kind {
        ember_ast::ImportKind::Module { path, alias } => (path, Some(alias)),
        ember_ast::ImportKind::Items { path, .. } => (path, None),
        ember_ast::ImportKind::Foreign { .. } => return None,
    };
    let names: Vec<String> = segments.iter().map(|segment| segment.name.to_string()).collect();
    if names.first().is_some_and(|first| first == ember_branding::STD_PACKAGE) {
        return None;
    }
    let (found, path, file) = packages.resolve_import(package, &names)?;
    if path != names {
        let span = segments.first().map_or(import.span, |segment| segment.span);
        let last = segments.last().cloned();
        if let (Some(alias), Some(last)) = (alias, last)
            && alias.is_none()
            && path.last() != names.last()
        {
            *alias = Some(last);
        }
        *segments = path.iter().map(|name| ember_ast::Ident { name: ember_span::Symbol::intern(name), span }).collect();
    }
    Some((found, path, file))
}

fn load_modules(
    root: ember_ast::Module,
    root_dir: &Path,
    map: &mut SourceMap,
    sink: &mut Sink,
) -> Vec<ember_typeck::LoadedModule> {
    // Without a resolution (an error the build reports itself), the root package alone.
    let packages = packages::resolve(root_dir).unwrap_or_else(|_| packages::Packages::alone(root_dir));
    load_package_modules(root, root_dir, &packages, map, sink)
}

fn load_package_modules(
    root: ember_ast::Module,
    root_dir: &Path,
    packages: &packages::Packages,
    map: &mut SourceMap,
    sink: &mut Sink,
) -> Vec<ember_typeck::LoadedModule> {
    let mut loaded: Vec<ember_typeck::LoadedModule> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut queue: Vec<(usize, Vec<String>, ember_ast::Module, Option<ember_typeck::ForeignImport>)> = Vec::new();
    // `[FFI-6]` — each `import c` loaded so far, by its header, folder and options.
    let mut foreign_known = cimport::Known::default();

    // `[MOD-5]` — source-backed prelude interfaces must exist even when the
    // user did not write an import. Loading a module does not make all of its
    // public items visible; type checking separately installs only the names
    // in the normative prelude list. Keep the root last so this LIFO worklist
    // still makes it module zero and preserves the command-line entry point.
    for module in ["core", "collections", "mem", "string"] {
        let names = vec![ember_branding::STD_PACKAGE.to_string(), module.to_string()];
        let key = names.join(".");
        let Some(file_path) = resolve_module(root_dir, &names) else {
            continue;
        };
        let file = match map.load(&file_path) {
            Ok(file) => file,
            Err(error) => {
                sink.emit(load_diagnostic(&error, ember_span::Span::DUMMY));
                continue;
            }
        };
        let text = map.file(file).text.clone();
        let lexed = ember_lexer::lex(file, &text, sink);
        let parsed = ember_parser::parse(file, &text, lexed.tokens, sink);
        mark("read and parse", &key);
        seen.insert(key);
        queue.push((usize::MAX, names, parsed, None));
    }
    queue.push((0, Vec::new(), root, None));

    while let Some((package, path, mut module, foreign)) = queue.pop() {
        // Every import this module names, before it is moved into the list: one of a package
        // (`[MOD-1]`) is already resolved, its path rewritten to the module's in the build.
        let mut wanted: Vec<(Vec<String>, ember_span::Span, bool, Option<(usize, PathBuf)>)> = Vec::new();
        for import in &mut module.imports {
            // `[FFI-6]` — `import c "header"`: a module written from the header, the import
            // rewritten to bind it.
            if matches!(import.kind, ember_ast::ImportKind::Foreign { .. }) {
                let importer = map.file(import.span.file).path.clone();
                let package_dir = packages.list.get(package).map(|found| found.dir.clone());
                let manifest = package_dir.as_ref().and_then(|dir| std::fs::read_to_string(dir.join(ember_branding::MANIFEST)).ok());
                let flags = manifest.as_deref().and_then(|text| manifest_list(text, "c", "flags")).unwrap_or_default();
                let base = package_dir.unwrap_or_else(|| importer.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf());
                if let Some(written) = cimport::load(import, &importer, &base, &flags, manifest.as_deref(), &mut foreign_known, map, sink) {
                    seen.insert(written.path.join("."));
                    for (child_path, child, foreign) in written.children {
                        seen.insert(child_path.join("."));
                        queue.push((package, child_path, child, Some(foreign)));
                    }
                    for (overlay_path, overlay, backed) in written.overlays {
                        seen.insert(overlay_path.join("."));
                        let graded = ember_typeck::ForeignImport { backed, ..Default::default() };
                        queue.push((package, overlay_path, overlay, Some(graded)));
                    }
                    queue.push((package, written.path, written.module, Some(written.foreign)));
                }
                continue;
            }
            let resolved = if package == usize::MAX { None } else { resolve_package_import(packages, package, import) };
            let segments = match &import.kind {
                ember_ast::ImportKind::Module { path, .. } => path,
                ember_ast::ImportKind::Items { path, .. } => path,
                // `import c "header.h"` is Phase 5's.
                ember_ast::ImportKind::Foreign { .. } => continue,
            };
            let names: Vec<String> = segments.iter().map(|s| s.name.to_string()).collect();
            // `[CLI-20]` — an import of nothing but 0.9.8's `with_views*`, which 0.9.9 removed.
            let removed = matches!(&import.kind, ember_ast::ImportKind::Items { items, glob: false, .. }
                if items.iter().all(|item| item.name.name.as_str().starts_with("with_views")));
            wanted.push((names, import.span, removed, resolved.map(|(found, _, file)| (found, file))));
        }

        // `[GRM-2]` (0.9.9) — only the entry file (the root, with an empty
        // path) may hold statements at file scope; elsewhere they are
        // `E0100`, and the implicit `main` built from them is dropped.
        if !path.is_empty() {
            if let Some(script) = module.script.take() {
                sink.emit(
                    ember_diag::Diagnostic::error(
                        ember_diag::codes::E0100,
                        script.first,
                        "a statement at file scope outside the entry file",
                    )
                    .help("move it into a function")
                    .note("only the file given to `ember run`/`ember build` is a script [GRM-2]"),
                );
                module.items.remove(script.main);
            }
        }
        loaded.push(ember_typeck::LoadedModule {
            path: path.clone(),
            module,
            package: if package == usize::MAX { None } else { Some(package) },
            foreign,
        });

        for (names, span, removed, resolved) in wanted {
            let key = names.join(".");
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key.clone());
            // `[MOD-1]` — a module of a package (this one's, or a dependency's): `names` is already
            // its path in the build.
            if let Some((found, file_path)) = resolved {
                let file = match map.load(&file_path) {
                    Ok(file) => file,
                    Err(error) => {
                        sink.emit(load_diagnostic(&error, span));
                        continue;
                    }
                };
                let text = map.file(file).text.clone();
                let lexed = ember_lexer::lex(file, &text, sink);
                let parsed = ember_parser::parse(file, &text, lexed.tokens, sink);
                check_safety_notes(&parsed, &lexed.comments, map, sink);
                mark("read and parse", &key);
                queue.push((found, names, parsed, None));
                continue;
            }
            // `[MOD-3]` — a standard module may be named without `std.`
            // (`import math`), unless the package has a module of that name (found above).
            let in_standard = |names: &[String]| {
                names.first().is_some_and(|first| first == ember_branding::STD_PACKAGE).then(|| resolve_module(root_dir, names)).flatten()
            };
            let mut names = names;
            if in_standard(&names).is_none() {
                let mut in_std = vec![ember_branding::STD_PACKAGE.to_string()];
                in_std.extend(names.iter().cloned());
                if in_standard(&in_std).is_some() {
                    if !seen.insert(in_std.join(".")) {
                        continue;
                    }
                    names = in_std;
                }
            }
            let Some(file_path) = in_standard(&names) else {
                // `[MOD-3]` — a module path that names no module is `E1060` at
                // the import, never accepted silently (F-004). A `std` module
                // the specification lists (Part XV) that this compiler has not
                // written yet is `E0900` (`[CLI-19]`).
                let path = names.join(".");
                if names.first().is_some_and(|f| f == ember_branding::STD_PACKAGE)
                    && names.len() == 2
                    && SPECIFIED_STD_MODULES.contains(&names[1].as_str())
                {
                    sink.emit(
                        ember_diag::Diagnostic::error(ember_diag::codes::E0900, span, format!("module `{path}` is not implemented yet"))
                            .note("the specification's standard library has it (Part XV); this compiler does not yet"),
                    );
                    continue;
                }
                let mut diagnostic = ember_diag::Diagnostic::error(
                    ember_diag::codes::E1060,
                    span,
                    format!("no module named '{path}'"),
                );
                let nearest = ember_typeck::nearest_names(&path, &known_module_paths(root_dir));
                if !nearest.is_empty() {
                    let listed: Vec<String> = nearest.iter().map(|module| format!("`{module}`")).collect();
                    diagnostic = diagnostic.help(format!("the nearest modules: {}", listed.join(", ")));
                }
                if removed {
                    diagnostic = diagnostic
                        .note("`with_views*` were removed in 0.9.9: a callable type gets fresh regions at each call [LT-7]")
                        .migrate("`with_views` import removed", span, "");
                }
                sink.emit(diagnostic);
                continue;
            };
            let file = match map.load(&file_path) {
                Ok(file) => file,
                Err(error) => {
                    sink.emit(load_diagnostic(&error, span));
                    continue;
                }
            };
            let text = map.file(file).text.clone();
            let lexed = ember_lexer::lex(file, &text, sink);
            let parsed = ember_parser::parse(file, &text, lexed.tokens, sink);
            mark("read and parse", &names.join("."));
            queue.push((usize::MAX, names, parsed, None));
        }
    }
    loaded
}

/// `[BLD-2]` / `[LT-40]` — build one canonical module-interface record per
/// loaded source module, reread it before borrow analysis consumes it, and
/// defer the write transaction until the completed MIR is verified. The
/// compiler still rechecks the whole graph today; the artifact establishes the
/// dependency identity and stale-record boundary required before later
/// incremental reuse can be sound.
fn prepare_callable_interface_cache(
    input: &Path,
    modules: &[ember_typeck::LoadedModule],
    declarations: &[ember_typeck::CallableDeclaration],
    map: &SourceMap,
    bodies: &mut [ember_mir::Body],
    types: &TypeTable,
    options: &Options,
) -> Result<PreparedInterfaceCache, String> {
    ember_analysis::install_callable_regions_all(bodies, types);

    let (inputs, import_visible_contracts) =
        module_interface_inputs(modules, declarations, map, bodies, types, options)?;
    let fresh = build_artifacts(&inputs, &compiler_identity()).map_err(|error| {
        format!("internal compiler error: [LT-40] cannot build interface artifact: {error}")
    })?;
    // The fresh record takes the same encode/decode path as a cache hit, so
    // every caller consumes the artifact representation even on its first
    // build rather than only on a later, incidental cache reuse.
    let fresh = round_trip_artifacts(&fresh).map_err(|error| {
        format!("internal compiler error: [LT-40] cannot round-trip interface artifact: {error}")
    })?;

    let target_root = options
        .out_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("target"));
    let profile_root = target_root.join(options.profile.name());
    let package_identity = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy()
        .replace('\\', "/");
    let directory = cache_directory(&profile_root, &package_identity);
    let prepared = prepare_interface_cache(&directory, &fresh).map_err(|error| {
        format!("internal compiler error: [LT-40] cannot load interface cache: {error}")
    })?;
    install_callable_metadata_from_artifacts(
        bodies,
        prepared.artifacts(),
        &import_visible_contracts,
    )?;
    Ok(prepared)
}

fn module_interface_inputs(
    modules: &[ember_typeck::LoadedModule],
    declarations: &[ember_typeck::CallableDeclaration],
    map: &SourceMap,
    bodies: &[ember_mir::Body],
    types: &TypeTable,
    options: &Options,
) -> Result<
    (
        Vec<ModuleInterfaceInput>,
        std::collections::BTreeMap<String, CallableInterfaceContract>,
    ),
    String,
> {
    use std::collections::{BTreeMap, BTreeSet};

    let module_names: BTreeMap<ember_span::FileId, String> = modules
        .iter()
        .map(|loaded| (loaded.module.span.file, module_identity(&loaded.path)))
        .collect();
    let known_modules: BTreeSet<String> = module_names.values().cloned().collect();
    let mut records_by_file: BTreeMap<
        ember_span::FileId,
        BTreeMap<String, CallableInterfaceContract>,
    > = BTreeMap::new();
    let import_visible_spans = import_visible_callable_spans(modules);
    let mut import_visible_contracts = BTreeMap::new();
    let mut declarations_by_span = BTreeMap::new();

    // The declaration surface is installed before looking at executable
    // bodies. A public generic therefore participates in interface identity
    // even when no concrete instantiation is emitted in this compilation.
    for declaration in declarations {
        if !module_names.contains_key(&declaration.span.file) {
            return Err(format!(
                "internal compiler error: [BLD-2] declaration `{}` has no source module for its interface artifact",
                declaration.symbol
            ));
        }
        let signature = declaration_signature(declaration, types)?;
        let contract = CallableInterfaceContract {
            signature,
            metadata: None,
            effects: None,
            inline_body: None,
        };
        match import_visible_contracts.get(&declaration.symbol) {
            // `[FFI-30]` — one C function declared by two modules (two imports of one header, or
            // two hand declarations of it) is one callable when the declarations agree.
            Some(existing) if *existing == contract => {}
            Some(_) => {
                return Err(format!(
                    "internal compiler error: [BLD-2] import-visible callable `{}` has more than one declaration contract",
                    declaration.symbol
                ));
            }
            None => {
                import_visible_contracts.insert(declaration.symbol.clone(), contract.clone());
            }
        }
        let records = records_by_file.entry(declaration.span.file).or_default();
        if records
            .insert(declaration.symbol.clone(), contract)
            .is_some()
        {
            return Err(format!(
                "internal compiler error: [BLD-2] source module declares callable `{}` twice",
                declaration.symbol
            ));
        }
        if declarations_by_span
            .insert(span_key(declaration.span), declaration)
            .is_some()
        {
            return Err(format!(
                "internal compiler error: [BLD-2] more than one callable declaration shares source span {:?}",
                declaration.span
            ));
        }
    }

    // `[BLD-2]` — the facts a caller may use beside a signature: each body's
    // effect set (`[EFF-4]`) and, for a function the inline header exports,
    // its body (`[CG-C-3b]`).
    let effect_sets = ember_analysis::effect_sets(bodies, types);
    let inline_symbols = ember_codegen_c::inline_header_symbols(bodies, types);
    for (index, body) in bodies.iter().enumerate() {
        let source_declaration = declarations_by_span.get(&span_key(body.span)).copied();
        if source_declaration.is_none() && !import_visible_spans.contains(&span_key(body.span)) {
            // The fresh MIR record remains installed for this compilation's
            // whole-program verification, but a private body is not part of
            // the import interface hash under `[BLD-2]`.
            continue;
        }
        // A generic source declaration has no executable body until it is
        // instantiated. The instantiated MIR symbol is deliberately not its
        // source declaration's identity, so it cannot replace the generic
        // interface contract with a concrete specialization.
        if source_declaration.is_some_and(|declaration| declaration.symbol != body.symbol) {
            continue;
        }
        let metadata = body.callable_regions.as_ref().ok_or_else(|| {
            format!(
                "internal compiler error: [MIR-REG-1] callable metadata missing before interface serialization for `{}`",
                body.symbol
            )
        })?;
        if !metadata.fingerprint_is_valid() {
            return Err(format!(
                "internal compiler error: [MIR-REG-1] callable metadata stale before interface serialization for `{}`",
                body.symbol
            ));
        }
        let signature = match source_declaration {
            Some(declaration) => declaration_signature(declaration, types)?,
            None => callable_signature(body, types)?,
        };
        if !module_names.contains_key(&body.span.file) {
            return Err(format!(
                "internal compiler error: [BLD-2] callable `{}` has no source module for its interface artifact",
                body.symbol
            ));
        }
        let inline_body = inline_symbols.contains_key(&body.symbol).then(|| {
            InterfaceHash::of_bytes(ember_mir::dump(std::slice::from_ref(body), types).as_bytes())
        });
        let contract = CallableInterfaceContract {
            signature,
            metadata: Some(metadata.clone()),
            effects: Some(effect_sets[index].each().map(|(_, name)| name.to_string()).collect()),
            inline_body,
        };
        if source_declaration.is_some() {
            let expected = import_visible_contracts.get_mut(&body.symbol).ok_or_else(|| {
                format!(
                    "internal compiler error: [BLD-2] callable body `{}` has no declaration contract",
                    body.symbol
                )
            })?;
            if expected.signature != contract.signature {
                return Err(format!(
                    "internal compiler error: [BLD-2] callable body `{}` disagrees with its resolved declaration signature",
                    body.symbol
                ));
            }
            *expected = contract.clone();
            let records = records_by_file.entry(body.span.file).or_default();
            let stored = records.get_mut(&body.symbol).ok_or_else(|| {
                format!(
                    "internal compiler error: [BLD-2] callable body `{}` has no module declaration record",
                    body.symbol
                )
            })?;
            *stored = contract;
        } else {
            // Members remain on the legacy body-derived path until their
            // declaration collector is widened too. They still retain a full
            // verified concrete signature and region contract; this fallback
            // must not erase existing visible-method coverage.
            let records = records_by_file.entry(body.span.file).or_default();
            if records.insert(body.symbol.clone(), contract.clone()).is_some()
                || import_visible_contracts
                    .insert(body.symbol.clone(), contract)
                    .is_some()
            {
                return Err(format!(
                    "internal compiler error: [BLD-2] import-visible member callable `{}` has more than one contract",
                    body.symbol
                ));
            }
        }
    }

    let implicit_prelude: Vec<String> = ["std.core", "std.collections", "std.mem"]
        .into_iter()
        .filter(|module| known_modules.contains(*module))
        .map(str::to_string)
        .collect();
    let package_config = format!("profile={}", options.profile.name());
    let mut layouts = type_layouts(modules, map);
    let mut inputs = Vec::with_capacity(modules.len());
    for loaded in modules {
        let module = module_identity(&loaded.path);
        let mut dependencies = BTreeSet::new();
        for import in &loaded.module.imports {
            let path = match &import.kind {
                ember_ast::ImportKind::Module { path, .. }
                | ember_ast::ImportKind::Items { path, .. } => path
                    .iter()
                    .map(|segment| segment.name.to_string())
                    .collect::<Vec<_>>()
                    .join("."),
                ember_ast::ImportKind::Foreign { .. } => continue,
            };
            if known_modules.contains(&path) && path != module {
                dependencies.insert(path);
            }
        }
        if !module.starts_with("std.") {
            for dependency in &implicit_prelude {
                if dependency != &module {
                    dependencies.insert(dependency.clone());
                }
            }
        }
        let language_version = loaded
            .module
            .directives
            .iter()
            .find(|directive| directive.name.name.is("language"))
            .and_then(|directive| match &directive.value {
                ember_ast::DirectiveValue::Text(value) => Some(value.clone()),
                ember_ast::DirectiveValue::ModuleAttribute(_) => None,
            })
            // `[VER-8]` — the one language; a directive can only repeat it.
            .unwrap_or_else(|| ember_parser::LANGUAGE_VERSION.to_string());
        // `[ECS-7]` — `std.ecs.components()` is written from the program's components.
        let mut source = map.file(loaded.module.span.file).text.clone();
        if module == "std.ecs" {
            source.push_str(&ember_typeck::component_registry_key(modules));
        }
        inputs.push(ModuleInterfaceInput {
            module,
            source,
            language_version,
            package_config: package_config.clone(),
            direct_dependencies: dependencies.into_iter().collect(),
            callables: records_by_file
                .remove(&loaded.module.span.file)
                .unwrap_or_default()
                .into_iter()
                .map(|(symbol, contract)| CallableInterfaceRecord { symbol, contract })
                .collect(),
            layouts: layouts.remove(&loaded.module.span.file).unwrap_or_default(),
        });
    }
    if let Some((file, records)) = records_by_file.into_iter().next() {
        return Err(format!(
            "internal compiler error: [BLD-2] {} callable interface record(s) have no loaded module for source file {}",
            records.len(),
            file.0
        ));
    }
    Ok((inputs, import_visible_contracts))
}

/// `[BLD-2]` — the layout of each import-visible type a module declares, by
/// its file and name: its kind, its fields as declared (name, type, default)
/// or its variants, and the attributes that place them (`repr`, `soa`, …). It
/// is read from the declaration, so it is the same in every program that
/// loads the module (a type the checker makes only when a program names it
/// would otherwise appear in some records and not others). A change to one
/// changes the module's interface hash, so its importers recheck.
fn type_layouts(
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
) -> std::collections::BTreeMap<ember_span::FileId, std::collections::BTreeMap<String, String>> {
    use ember_ast::{ItemKind, MemberKind, VisKind};
    use std::collections::BTreeMap;

    let mut out: BTreeMap<ember_span::FileId, BTreeMap<String, String>> = BTreeMap::new();
    for loaded in modules {
        let file = loaded.module.span.file;
        let text = &map.file(file).text;
        let source = |span: ember_span::Span| -> String {
            text.get(span.start as usize..span.end as usize).unwrap_or_default().split_whitespace().collect::<Vec<_>>().join(" ")
        };
        for item in &loaded.module.items {
            if item.vis.kind == VisKind::Private {
                continue;
            }
            let (kind, name, members, variants) = match &item.kind {
                ItemKind::Struct(decl) => ("struct", decl.name, &decl.members, Vec::new()),
                ItemKind::Class(decl) => ("class", decl.name, &decl.members, Vec::new()),
                ItemKind::Enum(decl) => ("enum", decl.name, &decl.members, decl.variants.iter().map(|variant| source(variant.span)).collect()),
                _ => continue,
            };
            let mut parts: Vec<String> = item.attrs.iter().map(|attr| source(attr.span)).collect();
            parts.push(kind.to_string());
            parts.extend(
                members
                    .iter()
                    .filter(|member| matches!(member.kind, MemberKind::Field(_)))
                    .map(|member| source(member.span)),
            );
            parts.extend(variants);
            out.entry(file).or_default().insert(name.name.as_str().to_string(), parts.join("; "));
        }
    }
    out
}

fn install_callable_metadata_from_artifacts(
    bodies: &mut [ember_mir::Body],
    artifacts: &[ModuleInterfaceArtifact],
    expected_contracts: &std::collections::BTreeMap<String, CallableInterfaceContract>,
) -> Result<(), String> {
    use std::collections::BTreeMap;

    let mut records: BTreeMap<String, CallableInterfaceContract> = BTreeMap::new();
    for artifact in artifacts {
        for (symbol, contract) in &artifact.callables {
            // `[FFI-30]` — one C function two modules declare alike is one callable.
            if records.insert(symbol.clone(), contract.clone()).is_some_and(|earlier| earlier != *contract) {
                return Err(format!(
                    "internal compiler error: [LT-40] callable `{symbol}` is present in more than one interface artifact"
                ));
            }
        }
    }
    for (symbol, expected) in expected_contracts {
        let Some(actual) = records.get(symbol) else {
            return Err(format!(
                "internal compiler error: [LT-40] interface artifact omits import-visible callable `{symbol}`"
            ));
        };
        if actual != expected {
            return Err(format!(
                "internal compiler error: [LT-40] interface artifact contract for `{symbol}` disagrees with current verified facts"
            ));
        }
    }
    for body in bodies {
        if let Some(contract) = records.get(&body.symbol) {
            // Public/package-visible contracts cross the artifact boundary.
            // Private bodies retain the freshly derived record that produced
            // this compilation, so all current whole-program checks remain
            // exact without publishing private implementation detail.
            if let Some(metadata) = &contract.metadata {
                body.callable_regions = Some(metadata.clone());
            }
        }
    }
    if let Some((symbol, _)) = records
        .into_iter()
        .find(|(symbol, _)| !expected_contracts.contains_key(symbol))
    {
        return Err(format!(
            "internal compiler error: [LT-40] interface artifact contains stale callable `{symbol}`"
        ));
    }
    Ok(())
}

/// Produce the resolved callable facts that can safely cross the first EMIF
/// signature boundary. MIR locals use `ref mut T` to implement a `mut T`
/// parameter, but the public contract records its ordinary declaration form:
/// mode `mut` plus the pointee `T`.
fn callable_signature(body: &ember_mir::Body, types: &TypeTable) -> Result<CallableSignature, String> {
    use ember_hir::Mode;
    use ember_types::TyKind;

    if body.param_modes.len() != body.arg_count {
        return Err(format!(
            "internal compiler error: [FN-1] callable `{}` has {} parameter mode(s) for {} argument(s)",
            body.symbol,
            body.param_modes.len(),
            body.arg_count
        ));
    }
    let parameters = body
        .args()
        .zip(&body.param_modes)
        .map(|((_, local), mode)| {
            let (mode, ty) = match mode {
                Mode::Borrow => (CallableParameterMode::Borrow, local.ty),
                Mode::Owned => (CallableParameterMode::Owned, local.ty),
                Mode::Mut => match types.kind(local.ty) {
                    TyKind::Ref { mutable: true, inner } => (CallableParameterMode::Mut, *inner),
                    _ => {
                        return Err(format!(
                            "internal compiler error: [FN-1] mutable callable `{}` has no mutable-reference parameter representation",
                            body.symbol
                        ));
                    }
                },
            };
            let ty = types.canonical_name(ty).map_err(|error| {
                format!(
                    "internal compiler error: [BLD-2] callable `{}` has a non-canonical parameter type: {error}",
                    body.symbol
                )
            })?;
            Ok(CallableParameter { mode, ty })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let result = types.canonical_name(body.return_ty()).map_err(|error| {
        format!(
            "internal compiler error: [BLD-2] callable `{}` has a non-canonical result type: {error}",
            body.symbol
        )
    })?;
    let borrows = body.borrows.as_ref().map(|positions| {
        let mut positions = positions.clone();
        positions.sort_unstable();
        positions.dedup();
        positions
    });
    Ok(CallableSignature {
        parameters,
        result,
        generics: Vec::new(),
        borrows,
        is_unsafe: body.is_unsafe,
        abi: body.abi.clone(),
        overflow: body.overflow.name().to_string(),
    })
}

/// Convert a type-checker declaration fact into EMIF's canonical source
/// contract. Bounds are sorted/deduplicated because their intersection has no
/// source-order semantics; generic parameter positions remain significant.
fn declaration_signature(
    declaration: &ember_typeck::CallableDeclaration,
    types: &TypeTable,
) -> Result<CallableSignature, String> {
    use ember_hir::Mode;

    let parameters = declaration
        .parameters
        .iter()
        .map(|parameter| {
            let mode = match parameter.mode {
                Mode::Borrow => CallableParameterMode::Borrow,
                Mode::Mut => CallableParameterMode::Mut,
                Mode::Owned => CallableParameterMode::Owned,
            };
            let ty = match &parameter.ty {
                ember_typeck::CallableDeclarationType::Resolved(ty) => {
                    types.canonical_name(*ty).map_err(|error| {
                        format!(
                            "internal compiler error: [BLD-2] declaration `{}` has a non-canonical parameter type: {error}",
                            declaration.symbol
                        )
                    })?
                }
                ember_typeck::CallableDeclarationType::Canonical(identity) => {
                    if identity.is_empty() {
                        return Err(format!(
                            "internal compiler error: [BLD-2] declaration `{}` has an empty canonical parameter identity",
                            declaration.symbol
                        ));
                    }
                    identity.clone()
                }
            };
            Ok(CallableParameter { mode, ty })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let result = types.canonical_name(declaration.result).map_err(|error| {
        format!(
            "internal compiler error: [BLD-2] declaration `{}` has a non-canonical result type: {error}",
            declaration.symbol
        )
    })?;
    let generics = declaration
        .generics
        .iter()
        .map(|generic| {
            let mut bounds = generic.bounds.clone();
            bounds.sort();
            bounds.dedup();
            let callable = generic
                .callable
                .as_ref()
                .map(|bound| -> Result<CallableGenericCallableBound, String> {
                    let parameters = bound
                        .parameters
                        .iter()
                        .map(|parameter| {
                            let mode = match parameter.mode {
                                ember_types::FnParamMode::Borrow => CallableParameterMode::Borrow,
                                ember_types::FnParamMode::Mut => CallableParameterMode::Mut,
                                ember_types::FnParamMode::Owned => CallableParameterMode::Owned,
                            };
                            let ty = types.canonical_name(parameter.ty).map_err(|error| {
                                format!(
                                    "internal compiler error: [BLD-2] declaration `{}` has a non-canonical Callable bound parameter: {error}",
                                    declaration.symbol
                                )
                            })?;
                            Ok(CallableParameter { mode, ty })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    let result = types.canonical_name(bound.result).map_err(|error| {
                        format!(
                            "internal compiler error: [BLD-2] declaration `{}` has a non-canonical Callable bound result: {error}",
                            declaration.symbol
                        )
                    })?;
                    Ok(CallableGenericCallableBound {
                        parameters,
                        result,
                        once: bound.once,
                        latebound: bound.latebound,
                        is_unsafe: bound.is_unsafe,
                    })
                })
                .transpose()?;
            Ok(CallableGenericParameter { bounds, callable })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let borrows = declaration.borrows.as_ref().map(|positions| {
        let mut positions = positions.clone();
        positions.sort_unstable();
        positions.dedup();
        positions
    });
    Ok(CallableSignature {
        parameters,
        result,
        generics,
        borrows,
        is_unsafe: declaration.is_unsafe,
        abi: declaration.abi.clone(),
        overflow: declaration.overflow.name().to_string(),
    })
}

/// `[MOD-2]` / `[BLD-2]` — spans of callable declarations observable by an
/// importing module. `pub(package)` is deliberately included: it is part of
/// this package's import graph even though it is absent from a dependant's
/// public API. Module-private item and member bodies must not perturb the
/// interface hash.
fn import_visible_callable_spans(
    modules: &[ember_typeck::LoadedModule],
) -> std::collections::BTreeSet<(ember_span::FileId, u32, u32)> {
    use ember_ast::{ItemKind, VisKind};

    let mut spans = std::collections::BTreeSet::new();
    for loaded in modules {
        for item in &loaded.module.items {
            match &item.kind {
                ItemKind::Fn(_) if item.vis.kind != VisKind::Private => {
                    spans.insert(span_key(item.span));
                }
                // A member is import-visible only through a visible named
                // type/interface. `extend` contributes public members to its
                // target's existing interface, so its own item visibility is
                // not a second gate.
                ItemKind::Struct(decl) if item.vis.kind != VisKind::Private => {
                    add_import_visible_members(&mut spans, &decl.members);
                }
                ItemKind::Class(decl) if item.vis.kind != VisKind::Private => {
                    add_import_visible_members(&mut spans, &decl.members);
                }
                ItemKind::Enum(decl) if item.vis.kind != VisKind::Private => {
                    add_import_visible_members(&mut spans, &decl.members);
                }
                ItemKind::Interface(decl) if item.vis.kind != VisKind::Private => {
                    add_import_visible_members(&mut spans, &decl.members);
                }
                ItemKind::Extend(decl) => add_import_visible_members(&mut spans, &decl.members),
                _ => {}
            }
        }
    }
    spans
}

fn add_import_visible_members(
    spans: &mut std::collections::BTreeSet<(ember_span::FileId, u32, u32)>,
    members: &[ember_ast::Member],
) {
    for member in members {
        if member.vis.kind != ember_ast::VisKind::Private
            && matches!(member.kind, ember_ast::MemberKind::Fn(_))
        {
            spans.insert(span_key(member.span));
        }
    }
}

fn span_key(span: ember_span::Span) -> (ember_span::FileId, u32, u32) {
    (span.file, span.start, span.end)
}

fn module_identity(path: &[String]) -> String {
    if path.is_empty() {
        "root".to_string()
    } else {
        path.join(".")
    }
}

/// Whether the C is for MSVC's `cl`: `--cc`, else the variable `cc_var()`
/// names, else the compiler the build would find first.
fn c_for_msvc(options: &Options) -> bool {
    let requested = options
        .cc
        .clone()
        .or_else(|| std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty()));
    match requested.as_deref() {
        Some("msvc") => true,
        Some("clang" | "clang-cl" | "gcc") => false,
        requested => {
            matches!(ember_build::Toolchain::detect(requested), Ok(ember_build::Toolchain::Msvc { cl, .. })
                if cl.file_stem().is_some_and(|stem| stem.eq_ignore_ascii_case("cl")))
        }
    }
}

/// ADR-098 — whether the C is for gcc: asked for (`--cc gcc` or the
/// variable), or the compiler found when none is asked for.
fn c_for_gcc(options: &Options) -> bool {
    let requested = options
        .cc
        .clone()
        .or_else(|| std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty()));
    match requested.as_deref() {
        Some("gcc") => true,
        Some("msvc" | "clang" | "clang-cl") => false,
        requested => matches!(ember_build::Toolchain::detect(requested), Ok(ember_build::Toolchain::Gcc(_))),
    }
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
    if names
        .first()
        .is_some_and(|f| f == ember_branding::STD_PACKAGE)
    {
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

/// `[FFI-26]` — derive a safe single filename for the generated public header.
/// A manifest's package name wins; a manifestless file is its own package.
fn export_package_name(input: &Path) -> Result<String, String> {
    let declared = nearest_manifest_text(input)?
        .as_deref().and_then(|manifest| manifest_string(manifest, "package", "name"));
    let name = declared.or_else(|| input.file_stem().and_then(|stem| stem.to_str()).map(str::to_owned))
        .ok_or_else(|| "cannot derive a package name for the generated header".to_string())?;
    let device_stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved_device = matches!(device_stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (device_stem.len() == 4
            && (device_stem.starts_with("COM") || device_stem.starts_with("LPT"))
            && device_stem.as_bytes()[3].is_ascii_digit()
            && device_stem.as_bytes()[3] != b'0');
    if name.is_empty() || matches!(name.as_str(), "." | "..")
        || name.ends_with('.') || name.ends_with(' ') || reserved_device
        || name.chars().any(|ch| ch.is_control() || matches!(ch,
            '/' | '\\' | ':' | '"' | '<' | '>' | '|' | '?' | '*')) {
        return Err(format!("package name `{name}` is not a single safe header filename component"));
    }
    Ok(name)
}

fn nearest_manifest_path(input: &Path) -> Option<PathBuf> {
    let mut directory = input.parent();
    while let Some(candidate) = directory {
        let path = candidate.join(ember_branding::MANIFEST);
        if path.is_file() { return Some(path); }
        directory = candidate.parent();
    }
    None
}

fn nearest_manifest_text(input: &Path) -> Result<Option<String>, String> {
    nearest_manifest_path(input).map(|path| std::fs::read_to_string(&path).map_err(|error|
        format!("cannot read package manifest `{}`: {error}", path.display()))).transpose()
}

/// `[DET-7]`, `[BLD-13]` — `--build-id`: the hash of a build's inputs, the same wherever the same
/// program is built the same way: this compiler (its own bytes), the profile, the target, the C
/// compiler by its version, the runtime's sources, and every source the build read (the manifest
/// among them) by its path in its package (`SourceMap::artefact_path`) and its text.
fn build_id(map: &SourceMap, profile: Profile, toolchain: &Toolchain, runtime: &Path) -> Result<String, String> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut part = |data: &[u8]| {
        bytes.extend_from_slice(&(data.len() as u64).to_le_bytes());
        bytes.extend_from_slice(data);
    };
    part(b"ember build id v1");
    let compiler = std::env::current_exe().and_then(std::fs::read).map_err(|error| format!("cannot read the compiler: {error}"))?;
    part(ember_build::content_hash(&compiler).as_bytes());
    part(profile.name().as_bytes());
    part(format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS).as_bytes());
    part(toolchain.version().as_bytes());
    let mut runtime_files: Vec<PathBuf> = ["include", "src"].iter()
        .filter_map(|dir| std::fs::read_dir(runtime.join(dir)).ok())
        .flat_map(|entries| entries.filter_map(|entry| entry.ok().map(|entry| entry.path())))
        .filter(|path| path.is_file())
        .collect();
    runtime_files.sort();
    for file in runtime_files {
        part(file.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default().as_bytes());
        part(&std::fs::read(&file).map_err(|error| format!("cannot read `{}`: {error}", file.display()))?);
    }
    let mut sources: Vec<(String, &str)> = map.files().map(|file| (map.artefact_path(file.id), file.text.as_str())).collect();
    sources.sort();
    for (path, text) in sources {
        part(path.as_bytes());
        part(text.as_bytes());
    }
    Ok(ember_build::content_hash(&bytes))
}

fn package_source_root(input: &Path) -> PathBuf {
    // A bare file name's parent is the empty path: its folder is the working one.
    let fallback = input.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new(".")).to_path_buf();
    let Some(manifest) = nearest_manifest_path(input) else { return fallback };
    let Some(package_root) = manifest.parent() else { return fallback };
    let source_root = package_root.join("src");
    let inside = std::fs::canonicalize(input).ok().zip(std::fs::canonicalize(&source_root).ok())
        .is_some_and(|(input, source_root)| input.starts_with(source_root));
    if inside { source_root } else { fallback }
}

fn compile(input: &Path, command: &str, options: &Options) -> Result<ExitCode, String> {
    let started = std::time::Instant::now();
    if options.timings.is_some() {
        start_timings();
    }
    let mut map = SourceMap::new();
    let mut sink = Sink::new();
    sink.deny_warnings = options.deny_warnings;
    let file = match map.load(input) {
        Ok(file) => file,
        Err(error @ LoadError::NotUtf8(_)) => {
            sink.emit(load_diagnostic(&error, Span::DUMMY));
            return Ok(finish(&sink, &map, options));
        }
        Err(error) => return Err(error.to_string()),
    };
    let source = map.file(file).text.clone();

    // Lex.
    let lexed = ember_lexer::lex(file, &source, &mut sink);
    mark("lex", "main");
    if options.emit.as_deref() == Some("tokens") {
        for token in &lexed.tokens {
            println!("{:?} {:?}", token.kind, token.span);
        }
        return Ok(finish(&sink, &map, options));
    }

    // Parse.
    let module = ember_parser::parse(file, &source, lexed.tokens, &mut sink);
    check_safety_notes(&module, &lexed.comments, &map, &mut sink);
    mark("parse", "main");
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
    let root_dir = package_source_root(input);
    let lint_settings = manifest_lint_settings(&root_dir, &mut map, &mut sink);
    let reload = reload_mode(&lint_settings, options, &mut sink)?;
    // `ember test` builds the package's tests as a program of their own, without reload.
    let reload = if command == "test" { ReloadMode::None } else { reload };
    // `[MOD-1]`, `[MAN-2]` — the package's dependencies, and its lock.
    let packages = packages::resolve(&root_dir)?;
    packages::settle_lock(&packages, options.locked)?;
    let mut modules = load_package_modules(module, &root_dir, &packages, &mut map, &mut sink);
    // `[BLD-13]` — what the program's source locations are written relative to: its package's
    // directory, each dependency's (by its name), the standard library's (`std`).
    let mut roots: Vec<(PathBuf, String)> = packages.list.iter().enumerate()
        .map(|(index, package)| (package.dir.clone(), if index == 0 { String::new() } else { package.name.clone() }))
        .collect();
    if let Some(std_root) = ember_branding::std_root() {
        roots.push((std_root, ember_branding::STD_PACKAGE.to_string()));
    }
    map.set_artefact_roots(roots);
    // Annex B — what a reloadable program's reload needs written into it before it is checked: its
    // fields' defaults as functions, `Old<Name>` from the previous build's schema.
    let reload_bin = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target")).join(options.profile.name()).join("bin");
    let reload_stem = input.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
    let reload_additions = (reload != ReloadMode::None).then(|| {
        let previous = std::fs::read_to_string(reload_bin.join(format!("{reload_stem}.schema.json")))
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok());
        reload::inject(&mut modules, &mut map, &mut sink, previous.as_ref())
    });
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // Check. `[CT-1]` — and check again with what the compile-time evaluator
    // answered, until nothing is left to evaluate; a pass that asks for
    // values says nothing (its errors are the next pass's too).
    let mut answers = ember_typeck::ComptimeAnswers::new();
    let mut reports = Vec::new();
    let mut blocked = Vec::new();
    let mut passes = 0;
    let (mut types, common, checked) = loop {
        passes += 1;
        let (mut types, common) = TypeTable::new();
        let mark = sink.mark();
        // [TYP-8] -- every profile defaults to panic; lexical attributes may
        // override that for arithmetic inside their declaration scope.
        let checked = ember_typeck::check(
            &modules,
            &mut types,
            &common,
            &mut sink,
            overflow_policy(options.profile),
            lint_settings.return_intersection,
            options.profile == Profile::Debug,
            &answers,
            lint_settings.realtime,
        );
        if checked.comptime.is_empty() {
            // `[TCB-1]` — under `--require-evidence` a grade claimed without evidence is an error.
            if options.require_evidence {
                for mut diagnostic in sink.take() {
                    if diagnostic.code == Some(codes::W5050) {
                        diagnostic.severity = ember_diag::Severity::Error;
                        diagnostic.code = Some(codes::E5050);
                        diagnostic.notes.push("a claimed grade needs its evidence under `--require-evidence` [TCB-1]".to_string());
                    }
                    sink.emit(diagnostic);
                }
            }
            break (types, common, checked);
        }
        sink.rollback(mark);
        let before = answers.len();
        let requests = checked.comptime.clone();
        if passes < COMPTIME_PASSES {
            let build = comptime_build(input, options)?;
            comptime::evaluate(checked, &mut types, &common, &map, &build.build(), &mut answers, &mut reports)?;
        }
        // What no pass can answer waits on itself, or on a function with
        // errors of its own: it fails, and the next pass is the last.
        if answers.len() == before {
            for request in &requests {
                answers.insert(request.key.clone(), ember_typeck::ComptimeAnswer::Failed);
                blocked.push(
                    Diagnostic::error(
                        codes::E6001,
                        request.span,
                        format!("{} cannot be evaluated: what it needs is itself computed from it while compiling", request.what),
                    )
                    .note("a compile-time value cannot depend on itself [CT-3]"),
                );
            }
        }
    };
    // `[MONO-3]` — a generic of the program's with more instances than the manifest's ceiling. The
    // standard library's generics are not the package's to budget.
    if let Some((ceiling, at)) = lint_settings.max_instantiations {
        let standard = format!("{}.", ember_branding::STD_PACKAGE);
        for entry in &checked.instantiations {
            if entry.instances.len() > ceiling && !entry.generic.starts_with(&standard) {
                let newest: Vec<&str> = entry.instances.iter().rev().take(3).map(String::as_str).collect();
                sink.emit(
                    Diagnostic::warning(
                        codes::W2220,
                        entry.span,
                        format!("`{}` has {} instances, more than `max_instantiations = {ceiling}`", entry.generic, entry.instances.len()),
                    )
                    .secondary(at, "the ceiling is set here")
                    .note(format!("its newest: {}", newest.join(", "))),
                );
            }
        }
    }
    // `[MONO-7]` — for each generic, whether its instances were specialised or share one body, and
    // why. With no ceiling set nothing is shared but what `@never_specialize` asks for (`[MONO-6]`).
    if options.generics_report {
        println!("Generics:");
        let standard = format!("{}.", ember_branding::STD_PACKAGE);
        for entry in checked.instantiations.iter().filter(|entry| !entry.generic.starts_with(&standard)) {
            let count = entry.instances.len();
            let line = match entry.sharing {
                Some(ember_typeck::Sharing::Shared) => format!("shared: its {count} instance(s) forward to one body (`@never_specialize`)"),
                Some(ember_typeck::Sharing::AlwaysSpecialize) => format!("specialised: {count} instance(s) (`@always_specialize`)"),
                None => format!("specialised: {count} instance(s) (no `@never_specialize`; this compiler shares no other body)"),
            };
            println!("  {}: {line}", entry.generic);
        }
    }
    // `[MONO-2]` — each generic's instances and the time checking them took.
    if options.instantiation_report {
        println!("Instantiations:");
        for entry in &checked.instantiations {
            println!("  {}: {} instance(s), {:.3} ms", entry.generic, entry.instances.len(), entry.elapsed.as_secs_f64() * 1000.0);
            for instance in &entry.instances {
                println!("    {instance}");
            }
        }
    }
    let only_these = !sink.has_errors();
    for report in reports {
        sink.emit(report);
    }
    // A request that waited on a function with errors is explained by them.
    if only_these {
        for report in blocked {
            sink.emit(report);
        }
    }
    let program = &checked.program;
    mark("check", "all modules");
    // `[WK-5]`–`[WK-7]` — class-field ownership cycles are a package-visible
    // lint over resolved types. It is diagnostic-only and therefore runs
    // before any MIR transformation can obscure the declared strong edges.
    if !sink.has_errors() {
        ember_analysis::lint_strong_cycles(&types, &mut sink);
    }
    if options.emit.as_deref() == Some("hir") {
        print!("{}", ember_hir::dump(program, &types));
        return Ok(finish(&sink, &map, options));
    }
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // Lower. `ember check` runs the MIR analyses too — Part XIX §1 defines it
    // as "type-check + borrow-check without codegen", so it cannot stop here.
    let mut bodies = ember_mir::lower(program, &types, &common, &map);
    let _ = &mut types;
    // `[TST-3]` — the package's tests (not the standard library's), in the
    // order the test binary numbers them. The binary's entry calls each, so
    // none is inlined into another caller and removed.
    let tests: Vec<&ember_hir::TestFn> = program.tests.iter()
        .filter(|test| !test.name.as_str().starts_with(&format!("{}.", ember_branding::STD_PACKAGE)))
        .collect();
    if command == "test" {
        for body in bodies.iter_mut().filter(|body| tests.iter().any(|test| test.symbol == body.symbol)) {
            body.inline.never = true;
        }
    }
    // `[COST-1]` — an implicit `clone` nothing calls is not emitted.
    ember_mir::prune_unused_implicit(&mut bodies, &types, &checked.std_instances);
    // Annex B — which functions the image reaches through thunks; `[HR-9a]`, none is inlined.
    let reload_image = (reload != ReloadMode::None).then(|| {
        let package = input.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
        reload::plan(reload, &package, &modules, &map, &bodies, reload_additions.as_ref().expect("made with the mode"))
    });
    if let Some(image) = &reload_image {
        for body in bodies.iter_mut().filter(|body| image.reloadable.contains(&body.symbol)) {
            body.inline.never = true;
        }
        reload::lint_noreload_loops(&bodies, image, &mut sink);
    }
    mark("lower", "all modules");
    if cfg!(debug_assertions) {
        ember_mir::verify::verify_all(&bodies);
    }
    // Definite initialisation (Part XVIII §4.6) runs on MIR, before any
    // optimisation could remove the read it is looking for.
    let initialization_facts = ember_analysis::analyze_definite_init_all(&bodies);
    // `[IMP-7]` — the fact verifier proves that the canonical initialization
    // fixpoint describes this initial MIR before diagnostics consume it and
    // before drop elaboration transforms the CFG.
    ember_analysis::verify_initialization_facts_all(&bodies, &initialization_facts);
    ember_analysis::check_definite_init_all_with_facts(&bodies, &initialization_facts, &mut sink);
    // `[LAY-2]` — a reference to a field of a `@packed` value is `E2170`.
    ember_analysis::check_packed_references_all(&bodies, &types, &mut sink);
    // `[OWN-3]`, Part XVIII §4.9 — moves are tracked, a use after a move is
    // `E3040`, and the drops lowering inserted are removed where the value was
    // moved away or made conditional on a drop flag where it may have been.
    ember_analysis::elaborate_drops_all(&mut bodies, &types, &mut sink);
    // `[LNT-3]` — `L1001`/`L1002` are emitted by `build` and `check`, not only
    // by `ember lint`. A lint that fires on a separate command does not close
    // the footgun `[GRM-4]` opens.
    // `[MIR-REG-1]` / `[LT-40]` — callable summaries are produced from the
    // post-drop MIR, round-trip through the canonical module-interface
    // artifact, then become the only contracts borrow checking consumes. The
    // cache write itself waits until the complete semantic verification below
    // succeeds, so an erroneous program never overwrites a verified record.
    let interface_cache = prepare_callable_interface_cache(
        input,
        &modules,
        &checked.callable_declarations,
        &map,
        &mut bodies,
        &types,
        options,
    )?;
    // `[CLS-7a]` — reject a class destructor's own handle entering visible
    // storage before code generation can expose a resurrection path.
    ember_analysis::check_drop_self_escapes_all(&bodies, &types, &mut sink);
    // `[FFI-21a]` — a function C may call returns no pointer into a
    // parameter C passes as a copy.
    ember_analysis::check_c_entry_returns_all(&bodies, &types, &mut sink);
    // `[FFI-36b]` — one foreign object adopted twice through one local (`E5068`).
    ember_analysis::check_adoptions_all(&bodies, &mut sink);
    // Part XVIII §4.7 — the NLL borrow checker runs on MIR after drop
    // elaboration, so the drops it sees are the ones that will exist.
    ember_analysis::check_all_with_installed_callable_regions(&bodies, &types, &mut sink);
    // `[DIA-8]` — read the loans where the checker read them.
    if let Some(line) = options.borrow_report {
        report(&sink, &map, options);
        print_borrow_report(&map, file, line, &bodies, &types);
        return Ok(ExitCode::SUCCESS);
    }
    // `[CORO-6]`, `[CORO-12]`, `[CORO-13]` — what a generator holds across a `yield`.
    ember_analysis::coroutines::check_all(&bodies, &types, &mut sink);
    // `[EXC-7]` — this advisory lint is opt-in. It consumes the explicit
    // method-duration dynamic-access intervals before later optimization may
    // elide or hoist an unrelated short interval.
    if lint_settings.long_term_access_across_dynamic_call && !sink.has_errors() {
        ember_analysis::lint_long_term_access_across_dynamic_calls_all(
            &bodies,
            &types,
            &mut sink,
        );
    }
    if !sink.has_errors() {
        verify_callable_regions_or_panic(&bodies, &types);
    }
    ember_analysis::check_unused_all(&bodies, &types, &mut sink);
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
        ember_mir::verify::verify_interface_upcasts_all(&bodies, &types);
    }
    mark("move, drop, borrow checks", "all modules");
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }
    // `[CORO-4]` — each generator's body, checked, becomes its frame's state
    // machine; what borrow checking read of the stubs is made again.
    if ember_analysis::coroutines::transform(&mut bodies, &mut types) && cfg!(debug_assertions) {
        ember_mir::verify::verify_all(&bodies);
    }
    // `[HEAP-5]` — borrow checking has established each mutable loan's exact
    // NLL region. Materialize the matching runtime access interval only after
    // that proof, so dynamic exclusivity follows the loan rather than a scope.
    ember_analysis::insert_shared_accesses_all(&mut bodies, &types);
    // `[EXC-3]`/`[EXC-3a]` — remove only the access intervals for which the
    // MIR proof establishes a unique, unescaped class handle. All other
    // intervals remain explicit runtime checks.
    ember_analysis::elide_static_accesses_all(&mut bodies, &types);
    // `[EXC-8]`–`[EXC-14]` — after static-elision proofs have removed their
    // intervals, conservatively turn a canonical stable class loop into one
    // checked preheader/postheader interval. Unknown loops remain per-access.
    ember_analysis::hoist_loop_accesses_all(&mut bodies, &types);
    // `[EXC-19]` — a `mut self` call to a quiet function is only checked. After
    // hoisting, which keeps a loop's repeated writes as one checked interval.
    ember_analysis::convert_quiet_calls_all(&mut bodies, &types);
    // `[EXC-3]`/`[EXC-3a]` — last, once every held interval is final: remove
    // the checks that nothing held anywhere in the program could make fail.
    ember_analysis::remove_never_firing_checks_all(&mut bodies, &types);
    // `[DSJ-5]` — an `assert_disjoint` comparison of views known apart is left out.
    ember_analysis::elide_known_disjoint_comparisons_all(&mut bodies, &types);
    // `[EFF-13]` — what is left of the dynamic aliasing checks is the verdict
    // a `@static_safe` function answers for, before inlining mixes callers'
    // and callees' checks.
    if ember_analysis::check_static_safe_all(&bodies, &mut sink) > 0 {
        return Ok(finish(&sink, &map, options));
    }
    // `[CLO-3]` — a known function passed to a callable parameter gets the
    // callee's own copy, calling it directly; first, so every copy is
    // optimised like any other body.
    ember_analysis::specialize_callable_arguments_all(&mut bodies);
    // `[EFF-5]`, `[EFF-15]` — the contracts, checked once on the effects left after the
    // profile-independent proofs and before inlining; a known function passed to a callable
    // parameter is called directly by now, so it contributes its own effects (`[EFF-2]`).
    if ember_analysis::check_effects_all(&bodies, &types, &mut sink) > 0 {
        return Ok(finish(&sink, &map, options));
    }
    // `[STD-6]`, `[BLD-11]` — a package's functions use only the layers it declares: the effects
    // of a layer it leaves out need what is not linked in this build.
    let file_packages: std::collections::HashMap<ember_span::FileId, usize> = modules.iter()
        .filter_map(|loaded| loaded.package.map(|package| (loaded.module.span.file, package)))
        .collect();
    let omitted = |body: &ember_mir::Body| -> Option<(ember_hir::Effects, String)> {
        let package = &packages.list[*file_packages.get(&body.span.file)?];
        let declared = package.layers.as_ref()?;
        let mut left_out = ember_hir::Effects::NONE;
        for (layer, effects) in ember_analysis::effects::LAYERS {
            if !package.uses(layer) {
                left_out |= effects;
            }
        }
        let declared: Vec<String> = declared.iter().map(|layer| format!("\"{layer}\"")).collect();
        Some((left_out, declared.join(", ")))
    };
    if ember_analysis::effects::check_layers_all(&bodies, &types, &omitted, &mut sink) > 0 {
        return Ok(finish(&sink, &map, options));
    }
    // `[EFF-16]` — what the safety side table says of each function, read where the contracts
    // were checked: inlining below folds a small function into its callers.
    let safety_functions = safety_functions_json(&bodies, &types, &modules, reload_image.as_ref().map(|image| &image.reloadable));
    // `[COST-4]` — the costs the checks do not show, read before inlining folds functions together.
    let safety_costs = safety_costs_json(&bodies, &types, &modules, &map, &checked.contains_uses);
    // `[LNT-6]` — `ember lint`'s own lints, on the MIR the checks ran on, before inlining.
    if options.lint {
        let (standard, _) = standard_parts(&bodies, &modules);
        ember_analysis::lint_performance_all(&bodies, &types, &common, &standard, &lint_settings.perf, &mut sink);
        lint_foreign_contracts(&modules, &lint_settings.perf, &mut sink);
    }
    // `[THR-5]` — a `Scope` binding stays where its `with` put it.
    if ember_analysis::check_scope_moves_all(&bodies, &types, &mut sink) > 0 {
        return Ok(finish(&sink, &map, options));
    }
    // `[CLI-10]` — the engine report, gathered as the build goes: the allocation sites here,
    // before inlining moves one function's operations into another.
    let mut engine = options.engine_report.then(EngineReport::default);
    if let Some(engine) = engine.as_mut() {
        let (standard, standard_files) = standard_parts(&bodies, &modules);
        // Before inlining folds some of them away: a caller the report leaves out.
        engine.standard_symbols = bodies.iter().zip(&standard).filter(|(_, standard)| **standard).map(|(body, _)| body.symbol.clone()).collect();
        for site in ember_analysis::allocation_sites(&bodies, &types, &standard, &|file| standard_files.contains(&file)) {
            engine.allocations.push(format!("{} in {}: {}", map.location(site.span), site.function, site.operations.join(", ")));
        }
    }
    // `[PHIL-2]`, `[CLO-10]` — the allocation sites, read where the effects
    // were: before inlining moves one function's operations into another.
    if let Some(request) = &options.alloc_report {
        report(&sink, &map, options);
        print_alloc_report(input, &bodies, &types, &modules, &map, request)?;
        return Ok(ExitCode::SUCCESS);
    }
    // `[UNS-8]` — the calls each `unsafe` makes, read before inlining moves them.
    if let Some(request) = &options.tcb_report {
        report(&sink, &map, options);
        print_tcb_report(input, &bodies, &types, &modules, &map, request)?;
        return Ok(ExitCode::SUCCESS);
    }
    // `[CLI-12]`, `[DET-9]` — likewise.
    if let Some(request) = &options.effect_report {
        report(&sink, &map, options);
        print_effect_report(input, &bodies, &types, &modules, &map, request)?;
        return Ok(ExitCode::SUCCESS);
    }
    // `[CLI-13]` — the call graph the effects were computed on, before inlining folds it.
    if let Some(request) = &options.foreign_calls {
        report(&sink, &map, options);
        print_foreign_calls(input, &bodies, &types, &modules, &map, request)?;
        return Ok(ExitCode::SUCCESS);
    }
    // A function called from one place is inlined there, so the analyses
    // below see what its caller knows. For clang and gcc, only a function
    // holding a check the caller's facts could remove: they inline the rest
    // themselves, and given a generic `larger` already inlined, clang picks
    // the slower unsigned `max` (`cmovbe`) for values it proves non-negative
    // (ADR-081). MSVC's inliner is weaker, so for it every such function.
    let inlined = ember_analysis::inline_single_calls_all(&mut bodies, &types, !c_for_msvc(&options));
    if let Some(engine) = engine.as_mut() {
        engine.inlined.extend(inlined);
    }
    // D-571 — a call through a value that is a known function calls it, and
    // that function, now called from one place, is inlined there too.
    if ember_analysis::call_known_callables_all(&mut bodies, &types) > 0 {
        let inlined = ember_analysis::inline_single_calls_all(&mut bodies, &types, !c_for_msvc(&options));
        if let Some(engine) = engine.as_mut() {
            engine.inlined.extend(inlined);
        }
        ember_analysis::install_callable_regions_all(&mut bodies, &types);
    }
    // ADR-107 — MSVC keeps a struct whose address goes to a function in
    // memory, inlined or not: a `for` loop's step through a standard
    // iterator is inlined and the iterator becomes a plain local.
    if c_for_msvc(&options) {
        ember_analysis::inline_loop_steps_all(&mut bodies, &types);
    }
    // Both passes above change bodies after `[MIR-REG-1]`'s summaries were
    // made (a call through a parameter became a direct call; a call became
    // its callee's body): make them again from the MIR as it now is.
    ember_analysis::install_callable_regions_all(&mut bodies, &types);
    // `[RNG-4]` — on the final checks: remove every check the range facts
    // prove cannot fail, before the loop passes group what is left.
    // A branch they decide goes one way, and what only the other way read
    // is gone: `[MIR-REG-1]`'s summaries are made again from what is left.
    let (_, folds) = ember_analysis::remove_proven_checks_all(&mut bodies, &types, &common);
    if folds > 0 {
        ember_analysis::install_callable_regions_all(&mut bodies, &types);
    }
    // D-759, `[TXT-4]` — a text slice's bound that is a character boundary by construction (`0`, the
    // text's length, a cursor `next_char` stepped from one) is not checked again.
    ember_analysis::remove_known_boundary_checks_all(&mut bodies, &types);
    // `[OPT-2]` — last, on the final checks: a counted loop whose indices are
    // all provably in bounds at entry runs a copy without its bounds checks;
    // then `[SIMD-7]` groups the overflow checks of every loop in
    // vectorisable form.
    // A versioned copy writes an element of a `MutSpan` through its index where the checked loop
    // called the runtime, which the summaries count as a write of the view: make them again.
    if ember_analysis::version_bounds_checked_loops_all(&mut bodies, &types, &common) != (0, 0) {
        ember_analysis::install_callable_regions_all(&mut bodies, &types);
    }
    // `[SIMD-2]` — each `@simd` loop's verdict, now that it is final: an
    // `@simd(assert)` loop not in vectorisable form is `E4020`, naming the
    // first clause of `[SIMD-5]` it fails; the report says the same of each.
    let simd_report = simd_verdicts(&bodies, &map, &mut sink);
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }
    // `[EXC-8]` — a dynamic exclusivity check every turn of a loop makes on
    // an unchanged handle, where nothing in the loop can change an access,
    // runs once before it; after versioning, so each copy of a loop is done.
    ember_analysis::hoist_invariant_checks_all(&mut bodies, &types);
    // `[RC-3]` — on the final MIR: a handle copied from a list element that
    // the list keeps alive for the handle's whole life is not counted.
    ember_analysis::mark_uncounted_handles_all(&mut bodies, &types);
    // `[RC-3]`: a hidden owner transfers only to an owner that survives its
    // original drop. An owned callee is not such a survival guarantee.
    ember_analysis::transfer_temporary_owners_all(&mut bodies, &types);
    // A hidden local that only copies another is read from that one, so no
    // copy stands between the loops of a nest; then the pure work a loop's
    // turn begins with whose inputs the loop cannot change (the view an
    // inner loop iterates) runs once, before the loop.
    ember_analysis::propagate_copies_all(&mut bodies, &types);
    ember_analysis::narrow_widened_reads_all(&mut bodies, &types);
    ember_analysis::hoist_invariant_views_all(&mut bodies, &types);
    // A counted loop that pushes onto a list on every turn asks for the room
    // before it, as a hint that changes nothing but the capacity (never done
    // in a program that asks a list for its `capacity()`).
    ember_analysis::reserve_pushed_lists_all(&mut bodies, &types);
    // For MSVC: a loop nest over separate lists goes into a function of its
    // own whose list parameters are `restrict` (ADR-079); MSVC reorders such
    // a nest only then. For gcc: a loop over two lists or more that writes
    // one, which gcc vectorises only then (ADR-098). clang does both where
    // the loop is; for it, and for MSVC's single loops, only a loop over
    // views `assert_disjoint` proved apart moves (`[DSJ-3]`).
    let kernel_target = if c_for_msvc(&options) {
        Some(ember_analysis::KernelTarget::Msvc)
    } else if c_for_gcc(&options) {
        Some(ember_analysis::KernelTarget::Gcc)
    } else {
        Some(ember_analysis::KernelTarget::Clang)
    };
    if command != "check"
        && let Some(target) = kernel_target
        && ember_analysis::outline_list_kernels_all(&mut bodies, &mut types, &common, target) > 0
    {
        ember_analysis::install_callable_regions_all(&mut bodies, &types);
    }
    // For clang and gcc: a list an innermost loop only pushes onto is kept in
    // a local for the loop and written back after it, so the C compiler keeps
    // its buffer and length in registers: it cannot see that the runtime's
    // buffer is apart from the list's own header (ADR-099). MSVC keeps such a
    // local in memory: the views-alive benchmark ran 8 times slower with it.
    if command != "check" && !c_for_msvc(&options) && ember_analysis::keep_pushed_lists_in_locals_all(&mut bodies, &types) > 0 {
        ember_analysis::install_callable_regions_all(&mut bodies, &types);
    }
    // Last, for C: a value a counted loop computes from its counter each
    // turn (`k * step + skip`) gets a running value of its own, as
    // hand-written C keeps one. clang and gcc vectorise such a loop at C's
    // speed only then; MSVC's vectoriser takes no second running value, so C
    // for MSVC keeps the value computed from the counter (the owner allows C
    // written for each compiler, 2026-09-28).
    if command != "check" && options.emit.as_deref() != Some("mir") && !c_for_msvc(&options) {
        ember_analysis::reduce_induction_values_all(&mut bodies, &types, &common);
    }
    // ADR-141 — last, for C: a short list or string a function makes, fills
    // and drops itself starts in a buffer of the function's own frame.
    if command != "check" && options.emit.as_deref() != Some("mir") {
        ember_analysis::give_stack_buffers_all(&mut bodies, &types);
    }
    if command == "check" {
        interface_cache
            .commit()
            .map_err(|error| error.to_string())?;
        report(&sink, &map, options);
        return Ok(ExitCode::SUCCESS);
    }

    if options.emit.as_deref() == Some("mir") {
        interface_cache
            .commit()
            .map_err(|error| error.to_string())?;
        print!("{}", ember_mir::dump(&bodies, &types));
        return Ok(finish(&sink, &map, options));
    }

    // Emit C. Source-backed prelude modules are always available to name
    // resolution, but their unused implementation bodies are not part of an
    // executable's observable generated program.
    if program.main.is_some() || command == "test" {
        let standard_files = modules
            .iter()
            .filter(|loaded| loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE))
            .map(|loaded| loaded.module.span.file)
            .collect();
        retain_referenced_standard_bodies(&mut bodies, &standard_files, &types);
    }
    let module_name = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    // `[TST-3]` — the test binary is a program of its own beside the package's.
    let module_name = if command == "test" { format!("{module_name}_test") } else { module_name };
    let manifest = nearest_manifest_text(input)?;
    let package_kind = manifest.as_deref().and_then(|text| manifest_string(text, "package", "kind"))
        .unwrap_or_else(|| "bin".to_string());
    let runtime_mode = manifest.as_deref().and_then(|text| manifest_string(text, "build", "runtime"))
        .unwrap_or_else(|| "static".to_string());
    let staticlib = command == "build" && package_kind == "staticlib";
    // `[FFI-28]` — a shared library: built as a library, with the module's entry points.
    let cdylib = command == "build" && package_kind == "cdylib";
    let library = staticlib || cdylib;
    // At the end of a program's `main` the process ends: a drop there that
    // only frees memory changes nothing observable (`[PHIL-5]`). Debug
    // builds keep every drop, which their leak check counts.
    if program.main.is_some() && !library && command != "test" && options.profile != Profile::Debug && !options.leak_check {
        ember_analysis::skip_exit_drops_all(&mut bodies, &types, &ember_branding::mangled("main"));
    }
    // Run after all body pruning and mutation. Retain the complete immutable
    // proof batch until the unconditional final verification below.
    let progress_certificates =
        ember_analysis::version_progress_reductions_all(&mut bodies, &types, &common);
    if !progress_certificates.is_empty() {
        ember_analysis::install_callable_regions_all(&mut bodies, &types);
    }
    verify_callable_regions_or_panic(&bodies, &types);
    let progress_violations = ember_analysis::verify_progress_reductions_all(
        &bodies, &types, &common, &progress_certificates,
    );
    if !progress_violations.is_empty() {
        panic!(
            "progress-reduction verification failed:\n{}",
            progress_violations
                .iter()
                .map(|v| format!("  {}: {}", v.body, v.message))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    // D-498 (ADR-155) — after the progress proof: this only copies a loop and
    // steps an ASCII byte the decoder reads anyway; it removes no check.
    let cold = progress_certificates.checked_fallbacks();
    if ember_analysis::step_ascii_runs_all(&mut bodies, &mut types, &common, &cold, kernel_target) > 0 {
        ember_analysis::install_callable_regions_all(&mut bodies, &types);
        verify_callable_regions_or_panic(&bodies, &types);
    }
    // `[IMP-7]` / `[VERIFY-3]` — verified MIR is a type-enforced backend
    // boundary. This check is unconditional and follows the final body-pruning
    // transformation, so release builds cannot emit stale or malformed MIR.
    let verified_mir =
        ember_mir::verify::for_codegen(&bodies, &types).unwrap_or_else(|violations| {
            panic!(
                "MIR code-generation verification failed:\n{}",
                violations
                    .iter()
                    .map(|v| format!("  {}: {}", v.body, v.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        });
    interface_cache
        .commit()
        .map_err(|error| error.to_string())?;
    if options.emit.is_none() {
        if command == "run" && package_kind != "bin" {
            return Err(format!("cannot run a `{package_kind}` package; use `ember build`"));
        }
        if command == "build" && !matches!(package_kind.as_str(), "bin" | "staticlib" | "cdylib") {
            return Err(format!("package kind `{package_kind}` is not implemented for `ember build`"));
        }
        if !matches!(runtime_mode.as_str(), "static" | "shared") {
            return Err(format!("`[build] runtime` is \"static\" or \"shared\", not `{runtime_mode}`"));
        }
        // `[FFI-31c]` — a shared runtime serves shared libraries (and hot reload); a program or a
        // static library links its own.
        if matches!(command, "build" | "run" | "test") && runtime_mode != "static" && reload_image.is_none() && !cdylib {
            return Err(format!("runtime mode `{runtime_mode}` is not implemented for this build artifact"));
        }
        if reload_image.is_some() && library {
            return Err(format!("a `{package_kind}` package is not reloadable: hot reload builds a program"));
        }
    }
    let header_requested = options.emit_header || options.emit.as_deref() == Some("header") ||
        (library && options.emit.is_none());
    let package_name = if header_requested || library { export_package_name(input)? } else { module_name.clone() };
    mark("analyses and passes", "all modules");
    // `[CLI-10]` — what the final MIR holds: count operations left in loops, foreign wrappers,
    // inlining hints, and the classes no handle copy ever shares.
    if let Some(engine) = engine.as_mut() {
        engine.gather_final(&bodies, &types, &modules, &map);
    }
    let emitted = ember_codegen_c::emit(
        verified_mir,
        &map,
        &module_name,
        &package_name,
        program.main.is_some() && !library && command != "test",
        // `[FFI-31]` — a shared library reports its leaks at shutdown in `debug`.
        options.leak_check || (cdylib && options.profile == Profile::Debug),
        library,
        cdylib,
        c_for_msvc(&options),
        (command == "test").then(|| tests.iter().map(|test| test.symbol.clone()).collect()),
        reload_image.as_ref(),
        command == "test" && options.ffi_evidence.is_some(),
    );
    mark("C generation", "all modules");
    // `[DSP-5]` — on standard error, so it never mixes with `--emit c`.
    // `[SIMD-2]` — then each `@simd` loop's verdict.
    if options.optimization_report {
        eprintln!("optimization report:");
        if emitted.optimization_report.is_empty() && simd_report.is_empty() {
            eprintln!("  none");
        }
        for line in emitted.optimization_report.iter().chain(&simd_report) {
            eprintln!("  {line}");
        }
    }
    if header_requested && !(library && options.emit.is_none()) {
        let target_dir = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target"));
        let layout = Layout::new(&target_dir, options.profile).map_err(|e| e.to_string())?;
        let header_path = layout.lib.join(format!("{package_name}.h"));
        if header_path.is_file() { std::fs::remove_file(&header_path).map_err(|e| e.to_string())?; }
        let header = emitted.header_source.as_ref().map_err(|error| error.clone())?;
        if header.needs_runtime_header {
            copy_runtime_header(&layout.lib)?;
        }
        std::fs::write(header_path, &header.source).map_err(|e| e.to_string())?;
    }
    if options.emit.as_deref() == Some("header") {
        return Ok(finish(&sink, &map, options));
    }
    if options.emit.as_deref() == Some("c") {
        if let Some(target_dir) = &options.out_dir {
            let layout = Layout::new(target_dir, options.profile).map_err(|e| e.to_string())?;
            std::fs::write(layout.c.join(format!("{module_name}.c")), &emitted.c_source)
                .map_err(|e| e.to_string())?;
            for (mode, source) in &emitted.relaxed_units {
                std::fs::write(layout.c.join(format!("{module_name}.{}.c", mode.name())), source)
                    .map_err(|e| e.to_string())?;
            }
        }
        print!("{}", emitted.c_source);
        // `[CG-C-11]` — each relaxed unit after the main one.
        for (_, source) in &emitted.relaxed_units {
            print!("{source}");
        }
        return Ok(finish(&sink, &map, options));
    }
    // `[CLI-2]` — the C compiled to object files, not linked: the program's
    // unit and each relaxed one (`[CG-C-11]`), under `<out-dir>/<profile>/obj`.
    if options.emit.as_deref() == Some("obj") {
        let target_dir = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target"));
        let layout = Layout::new(&target_dir, options.profile).map_err(|e| e.to_string())?;
        let requested = options
            .cc
            .clone()
            .or_else(|| std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty()));
        let toolchain = Toolchain::detect(requested.as_deref()).map_err(|e| e.to_string())?;
        let includes = vec![runtime_dir()?.join("include")];
        let object_ext = if matches!(&toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
        let mut units = vec![(format!("{module_name}.c"), emitted.c_source.clone(), None)];
        for (mode, source) in &emitted.relaxed_units {
            units.push((format!("{module_name}.{}.c", mode.name()), source.clone(), Some(*mode == ember_types::FpMode::Fast)));
        }
        for (name, source, relaxed) in units {
            let c_path = layout.c.join(&name);
            std::fs::write(&c_path, source).map_err(|e| e.to_string())?;
            let object = layout.obj.join(format!("{}.{object_ext}", name.trim_end_matches(".c")));
            let compiled = match relaxed {
                None => ember_build::compile_object(&toolchain, &c_path, &includes, &object, options.profile),
                Some(fast) => ember_build::compile_relaxed_object(&toolchain, &c_path, &includes, &object, options.profile, fast),
            };
            if let Err(error) = compiled {
                if report_float_mode_refused(&mut sink, &bodies, &error) {
                    return Ok(finish(&sink, &map, options));
                }
                return Err(error.to_string());
            }
            eprintln!("built {}", object.display());
        }
        return Ok(finish(&sink, &map, options));
    }
    if let Some(other) = &options.emit {
        return Err(format!("unknown `--emit` stage `{other}`"));
    }

    if program.main.is_none() && !library && command != "test" {
        return Err("this file declares no `main`, so there is nothing to run".to_string());
    }

    // Write, compile and link.
    let target_dir = options
        .out_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("target"));
    let layout = Layout::new(&target_dir, options.profile).map_err(|e| e.to_string())?;
    let c_path = layout.c.join(format!("{module_name}.c"));
    std::fs::write(&c_path, &emitted.c_source).map_err(|e| e.to_string())?;
    let safety_path = layout.inspect.join(format!("{module_name}.safety.json"));
    let safety_json = {
        let table = emitted.safety_json.trim_end();
        let table = table.strip_suffix('}').unwrap_or(table);
        format!("{table},\"functions\":{safety_functions},\"costs\":{safety_costs}}}\n")
    };
    std::fs::write(&safety_path, safety_json).map_err(|e| e.to_string())?;
    // `[RC-6]` — the count operations left inside loops, read from the same
    // final MIR the backend was given.
    let counts_path = layout.inspect.join(format!("{module_name}.counts.json"));
    std::fs::write(&counts_path, counts_json(&bodies, &types, &map)).map_err(|e| e.to_string())?;

    let runtime = runtime_dir()?;
    let exe_name = if cfg!(windows) {
        format!("{module_name}.exe")
    } else {
        module_name.clone()
    };
    let exe = layout.bin.join(exe_name);

    // `--cc`, else the variable `cc_var()` names (CI's compiler matrix, D-251).
    let requested = options
        .cc
        .clone()
        .or_else(|| std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty()));
    let toolchain = Toolchain::detect(requested.as_deref()).map_err(|e| e.to_string())?;
    // `[BLD-FFI-1b]` — the C++ projects' runtime switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL`, which
    // every translation unit of the target is compiled with.
    if matches!(toolchain, Toolchain::Msvc { .. }) {
        match cimport::inherited_runtime(&modules) {
            Ok(flags) => ember_build::inherit_msvc_runtime(flags),
            Err(diagnostic) => {
                sink.emit(diagnostic);
                return Ok(finish(&sink, &map, options));
            }
        }
    }
    if options.build_id {
        println!("{}", build_id(&map, options.profile, &toolchain, &runtime)?);
    }
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let includes = vec![runtime.join("include")];
    // Annex B — a reloadable program: the shared runtime, an image and the host that runs it. The
    // image holds the imports' shims and thunks (`[FFI-39a]`: a trampoline's object keeps calling
    // through the permanent thunks of the entries it was given).
    if let Some(image) = &reload_image {
        let c_toolchain = match manifest.as_deref().and_then(|text| manifest_string(text, "c", "compiler")) {
            Some(compiler) => Toolchain::detect(Some(&compiler)).map_err(|e| e.to_string())?,
            None => toolchain.clone(),
        };
        let foreign = cimport::build_objects(&modules, &c_toolchain, &toolchain, &layout.c, &layout.obj, options.profile)?;
        let built = match reload::build(&toolchain, &layout, &module_name, &c_path, &runtime_source, &includes,
            options.profile, image, &emitted.relaxed_units, &foreign, options.reload, options.explain, started)
        {
            Ok(built) => built,
            Err(reload::Failure::Other(message)) => return Err(message),
            // `[HR-12a]` — the process's header is fixed when it starts: an image built for the other
            // header cannot join it.
            Err(reload::Failure::Header { running, built }) => {
                sink.emit(
                    Diagnostic::error(codes::E9035, Span::DUMMY, format!(
                        "the running program's objects have {running}-byte headers, and this image's would have {built}"))
                        .note("whether reload migrates instances (`[build] reload`) is the whole process's to agree on: it fixes the object header [HR-12a]")
                        .help("build the program again with `ember build` and restart it"),
                );
                return Ok(finish(&sink, &map, options));
            }
        };
        mark("C compile and link", "all modules");
        // `[HR-18b]` — the running program's own plan of this image, applying nothing.
        if options.explain {
            report(&sink, &map, options);
            let (apply, text) = reload::explain(&layout.bin, &module_name, &built.image)?;
            println!("{}", if apply { "the reload would apply" } else { "the reload would be refused" });
            print!("{text}");
            return Ok(if apply { ExitCode::SUCCESS } else { ExitCode::FAILURE });
        }
        // `[HR-16]` — this build's schema, which the next build's `Old<Name>` structs are made from.
        if let Some(additions) = &reload_additions {
            let schema = serde_json::Value::Object(additions.schema.clone());
            std::fs::write(layout.bin.join(format!("{module_name}.schema.json")), schema.to_string()).map_err(|e| e.to_string())?;
        }
        report(&sink, &map, options);
        if command == "run" {
            return run_program(&built.host, options);
        }
        eprintln!("built {}", if options.reload { built.image.display() } else { built.host.display() });
        return Ok(ExitCode::SUCCESS);
    }
    // `[CG-C-11]` — a library's relaxed units, each compiled with its own flags into an object of its
    // own: the archive or the shared library takes them beside the package's main unit.
    let mut library_relaxed = Vec::new();
    if library {
        let object_ext = if matches!(&toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
        for (mode, source) in &emitted.relaxed_units {
            let unit = layout.c.join(format!("{module_name}.{}.c", mode.name()));
            std::fs::write(&unit, source).map_err(|e| e.to_string())?;
            let object = layout.obj.join(format!("{module_name}.{}.{object_ext}", mode.name()));
            let fast = *mode == ember_types::FpMode::Fast;
            let compiled = if cdylib {
                ember_build::compile_relaxed_library_object(&toolchain, &unit, &includes, &object, options.profile, fast)
            } else {
                ember_build::compile_relaxed_object(&toolchain, &unit, &includes, &object, options.profile, fast)
            };
            if let Err(error) = compiled {
                if report_float_mode_refused(&mut sink, &bodies, &error) {
                    return Ok(finish(&sink, &map, options));
                }
                return Err(error.to_string());
            }
            library_relaxed.push(object);
        }
    }
    // `[BLD-FFI-3]` — `[link] libs`, in the order written.
    // A path is the package's: relative to the directory of its manifest.
    let package_dir = nearest_manifest_path(input).and_then(|path| path.parent().map(Path::to_path_buf));
    let mut libraries: Vec<String> = manifest.as_deref().and_then(|text| manifest_list(text, "link", "libs")).unwrap_or_default()
        .into_iter()
        .map(|library| match &package_dir {
            Some(dir) if ember_build::cdylib::is_library_path(&library) && Path::new(&library).is_relative() =>
                dir.join(&library).to_string_lossy().into_owned(),
            _ => library,
        })
        .collect();
    // `[FFI-29]`, `[BLD-FFI-1]` — each `import c`'s shim and C sources, compiled by the `[c]`
    // section's compiler (the backend's by default) into objects of their own; `[BLD-FFI-3]` its
    // libraries after the manifest's, in the order written.
    let c_toolchain = match manifest.as_deref().and_then(|text| manifest_string(text, "c", "compiler")) {
        Some(compiler) => Toolchain::detect(Some(&compiler)).map_err(|e| e.to_string())?,
        None => toolchain.clone(),
    };
    let (foreign_objects, foreign_libraries) = cimport::build_objects(&modules, &c_toolchain, &toolchain, &layout.c, &layout.obj, options.profile)?;
    libraries.extend(foreign_libraries);
    library_relaxed.extend(foreign_objects.iter().cloned());
    // `[FFI-28]`, `[FFI-31]` — a shared library exporting only its exports and the module's entry
    // points; `[FFI-31c]` with `runtime = "shared"` it imports the runtime its process shares.
    if cdylib {
        let header = emitted.header_source.as_ref().map_err(|error| error.clone())?;
        let mut exports: Vec<String> = bodies.iter()
            .filter(|body| body.abi.is_some() && !body.is_extern_declaration && !body.is_abstract)
            .map(|body| body.symbol.clone())
            .collect();
        // `[FFI-28]`, `[ABI-1]` (ODR-131) — the entry points and the module protocol's marker.
        exports.extend([ember_branding::runtime("module_init"), ember_branding::runtime("module_shutdown"),
            format!("{} DATA", ember_branding::runtime(MODULE_PROTOCOL_MARKER))]);
        // `[FFI-26]` — each export table, its protocol version in its symbol, as data.
        for table in types.export_tables() {
            exports.push(format!("{} DATA", table.symbol()));
        }
        exports.sort();
        exports.dedup();
        let shared = if runtime_mode == "shared" {
            Some(ember_build::reload::shared_runtime(&toolchain, &runtime_source, &includes, options.profile, None,
                &ember_build::cache_root(), &layout.lib).map_err(|e| e.to_string())?)
        } else {
            None
        };
        let package_header = layout.lib.join(format!("{package_name}.h"));
        let built = ember_build::cdylib::link_cdylib(&toolchain, &package_name, &c_path, &runtime_source, &includes,
            options.profile, shared.as_ref(), &exports, &libraries, &library_relaxed, &layout.obj, &layout.lib)
            .map_err(|e| e.to_string())?;
        copy_runtime_header(&layout.lib)?;
        std::fs::write(&package_header, &header.source).map_err(|e| e.to_string())?;
        report(&sink, &map, options);
        eprintln!("built {}", built.library.display());
        return Ok(ExitCode::SUCCESS);
    }
    if staticlib {
        let header = emitted.header_source.as_ref().map_err(|error| error.clone())?;
        let object_ext = if matches!(&toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
        let package_object = layout.obj.join(format!("{module_name}.{object_ext}"));
        let runtime_object = layout.obj.join("runtime").join(format!("{}_rt.{object_ext}", ember_branding::SYMBOL_PREFIX));
        let package_archive = layout.lib.join(ember_build::archive_name(&toolchain, &package_name));
        let runtime_lib = runtime_artifact_dir(&layout.lib);
        let runtime_archive = runtime_lib.join(ember_build::archive_name(&toolchain,
            &format!("{}_rt", ember_branding::SYMBOL_PREFIX)));
        let package_header = layout.lib.join(format!("{package_name}.h"));
        let runtime_header = runtime_lib.join(ember_branding::runtime_header());
        let outputs = [&package_archive, &runtime_archive, &package_header, &runtime_header];
        for path in outputs { if path.is_file() { std::fs::remove_file(path).map_err(|e| e.to_string())?; } }
        let built = (|| -> Result<(), String> {
            ember_build::compile_object(&toolchain, &c_path, &includes, &package_object, options.profile)
                .map_err(|e| e.to_string())?;
            ember_build::compile_object(&toolchain, &runtime_source, &includes, &runtime_object, options.profile)
                .map_err(|e| e.to_string())?;
            let mut package_objects = vec![package_object];
            package_objects.extend(library_relaxed.iter().cloned());
            ember_build::archive_objects(&toolchain, &package_objects, &package_archive)
                .map_err(|e| e.to_string())?;
            ember_build::archive_objects(&toolchain, &[runtime_object], &runtime_archive)
                .map_err(|e| e.to_string())?;
            copy_runtime_header(&layout.lib)?;
            std::fs::write(&package_header, &header.source).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if let Err(error) = built {
            for path in outputs { let _ = std::fs::remove_file(path); }
            return Err(error);
        }
        report(&sink, &map, options);
        eprintln!("built {} and {}", package_archive.display(), runtime_archive.display());
        return Ok(ExitCode::SUCCESS);
    }
    // The runtime is compiled once per toolchain and profile, then linked as
    // an object; without one (MSVC), it is compiled with the program.
    let runtime_input = ember_build::runtime_object_for_executable(
        &toolchain,
        &runtime_source,
        &includes,
        options.profile,
        &ember_build::cache_root(),
    )
    .map_err(|e| e.to_string())?
    .unwrap_or(runtime_source);
    let mut sources = vec![c_path.clone(), runtime_input];
    sources.extend(foreign_objects);
    // `[FFI-37]` — the instrumentation is a unit of its own (it defines the C library's entry
    // points it intercepts off Windows).
    if let Some(instrument) = &emitted.ffi_instrument {
        let unit = layout.c.join(format!("{module_name}.ffi.c"));
        std::fs::write(&unit, &instrument.source).map_err(|e| e.to_string())?;
        sources.push(unit);
    }
    // `[CG-C-11]` — each relaxed unit compiled with its own flags.
    let object_ext = if matches!(&toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
    for (mode, source) in &emitted.relaxed_units {
        let unit = layout.c.join(format!("{module_name}.{}.c", mode.name()));
        std::fs::write(&unit, source).map_err(|e| e.to_string())?;
        let object = layout.obj.join(format!("{module_name}.{}.{object_ext}", mode.name()));
        if let Err(error) = ember_build::compile_relaxed_object_for_executable(
            &toolchain,
            &unit,
            &includes,
            &object,
            options.profile,
            *mode == ember_types::FpMode::Fast,
        ) {
            if report_float_mode_refused(&mut sink, &bodies, &error) {
                return Ok(finish(&sink, &map, options));
            }
            return Err(error.to_string());
        }
        sources.push(object);
    }
    // The C header and executable expose the same definitions. Root selected
    // ABI symbols, including renamed/imported exports, never declarations.
    // `[MNG-2]` — an export's object-file symbol, which is its alias's exact one.
    let mut retained_symbols: Vec<String> = bodies.iter()
        .filter(|body| body.abi.is_some()
            && !body.is_extern_declaration && !body.is_abstract)
        .map(|body| ember_branding::export_alias_symbol(&body.symbol).unwrap_or_else(|| body.symbol.clone()))
        .collect();
    if emitted.ffi_instrument.is_some() && !cfg!(windows) {
        retained_symbols.extend(ember_codegen_c::FFI_INTERPOSED.iter().map(|name| name.to_string()));
    }
    for table in types.export_tables() {
        retained_symbols.push(table.symbol());
    }
    retained_symbols.sort();
    retained_symbols.dedup();
    ember_build::compile_and_link_program(
        &toolchain,
        &LinkRequest {
            sources: &sources,
            include_dirs: &includes,
            output: exe.clone(),
            profile: options.profile,
            obj_dir: layout.obj.clone(),
        },
        &retained_symbols,
        &libraries,
    )
    .map_err(|e| e.to_string())?;
    mark("C compile and link", "all modules");
    // `[CLI-10]` — the checks the program keeps and why, and the C compiler's word on its loops.
    if let Some(engine) = engine.as_mut() {
        engine.gather_checks(&emitted.safety_json);
        engine.loops = match ember_build::vectorisation_report(&toolchain, &c_path, &includes, &layout.obj.join("report"), options.profile) {
            Ok(remarks) => remarks,
            Err(error) => vec![format!("not available: {error}")],
        };
    }
    // `[SIMD-6]` — after Ember's verdicts, the host C compiler's own report on
    // the program's unit, labelled as its opinion: its absence or a
    // disagreement is never an error.
    if options.optimization_report {
        eprintln!("host C compiler's vectorisation report ({}; its opinion only, Ember's verdict is above):", toolchain.name());
        match ember_build::vectorisation_report(&toolchain, &c_path, &includes, &layout.obj.join("report"), options.profile) {
            Ok(remarks) if remarks.is_empty() => eprintln!("  none"),
            Ok(remarks) => {
                for remark in remarks {
                    eprintln!("  {remark}");
                }
            }
            Err(error) => eprintln!("  not available: {error}"),
        }
    }

    report(&sink, &map, options);
    if let Some(engine) = &engine {
        engine.print(&toolchain);
    }

    if command == "test" {
        // `[FFI-37]` — what the instrumented binary counts, with what the program declares of it.
        let instrumented = emitted.ffi_instrument.as_ref().zip(options.ffi_evidence.as_ref()).map(|(instrument, dir)| {
            let package = input.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
            let mut run = foreign::InstrumentedRun::new(instrument, &bodies, dir.clone(), &package, &exe, toolchain.identity());
            run.imported = modules
                .iter()
                .filter_map(|loaded| loaded.foreign.as_ref().map(|foreign| (loaded.module.span.file, foreign.identities.clone())))
                .collect();
            run
        });
        return test_runner::run_tests(&exe, &tests, options, instrumented.as_ref(), &map);
    }
    if command == "run" {
        return run_program(&exe, options);
    }

    eprintln!("built {}", exe.display());
    Ok(ExitCode::SUCCESS)
}

/// `[HR-33]`, `[HR-31]` — `ember run --hot`: build the program under `debug`, run it, and while
/// it runs watch the package's sources; after each edit, build the next image and announce it,
/// which the program applies at its next `std.hot.checkpoint()`. Building happens here, in the
/// background of the program, so a checkpoint never waits for it. A build that fails leaves the
/// old code running. The program is told when ten seconds pass with no checkpoint reached.
fn run_hot(input: &Path, options: &Options) -> Result<ExitCode, String> {
    if options.profile != Profile::Debug {
        return Err("`ember run --hot` builds under `debug`".to_string());
    }
    let build = Options { emit: None, ..options_for_hot(options, false) };
    let code = compile(input, "build", &build)?;
    if code != ExitCode::SUCCESS {
        return Ok(code);
    }
    let bin = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target")).join(options.profile.name()).join("bin");
    let stem = input.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
    let host = bin.join(if cfg!(windows) { format!("{stem}.exe") } else { stem.clone() });
    let mut child = std::process::Command::new(&host)
        .args(&options.program_args)
        .env(format!("{}_HOT", ember_branding::SYMBOL_PREFIX.to_uppercase()), "1")
        .spawn()
        .map_err(|error| format!("cannot start `{}`: {error}", host.display()))?;
    let root = package_source_root(input);
    let mut seen = source_stamp(&root);
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            if let Some(how) = abnormal_end(&status) {
                eprintln!("error: {how}");
            }
            return Ok(match status.code() {
                Some(0) => ExitCode::SUCCESS,
                Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
                None => ExitCode::FAILURE,
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        let now = source_stamp(&root);
        if now != seen {
            seen = now;
            // A failed build is reported and changes nothing the program runs ([HR-41]).
            match compile(input, "build", &options_for_hot(options, true)) {
                Ok(_) => {}
                Err(message) => eprintln!("error: {message}"),
            }
        }
    }
}

/// The options of one of `ember run --hot`'s builds: the first, or the next image (`--reload`).
fn options_for_hot(options: &Options, reload: bool) -> Options {
    Options {
        profile: options.profile,
        cc: options.cc.clone(),
        out_dir: options.out_dir.clone(),
        json: options.json,
        deny_warnings: options.deny_warnings,
        reload,
        hot: true,
        ..Options::default()
    }
}

/// Every source file of a package, with its size and time of change: what `ember run --hot`
/// watches.
fn source_stamp(root: &Path) -> Vec<(PathBuf, u64, Option<std::time::SystemTime>)> {
    let mut out = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                if path.file_name().is_some_and(|name| name != "target" && !name.to_string_lossy().starts_with('.')) {
                    pending.push(path);
                }
            } else if path.extension().is_some_and(|extension| extension == ember_branding::SOURCE_EXT)
                || path.file_name().is_some_and(|name| name == ember_branding::MANIFEST)
            {
                out.push((path, meta.len(), meta.modified().ok()));
            }
        }
    }
    out.sort();
    out
}

/// `ember run`: the program, with its arguments, its status as the command's.
fn run_program(exe: &Path, options: &Options) -> Result<ExitCode, String> {
    let status = std::process::Command::new(exe)
        .args(&options.program_args)
        .status()
        .map_err(|e| e.to_string())?;
    // A status outside 0..=255 (a Windows exception arrives as a large
    // negative i32) or a Unix signal is a crash: clamping it into a u8
    // turned one into a clean exit, so it becomes a plain failure, and it
    // is said what stopped the program (D-464).
    if let Some(how) = abnormal_end(&status) {
        eprintln!("error: {how}");
    }
    Ok(match status.code() {
        Some(0) => ExitCode::SUCCESS,
        Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        None => ExitCode::FAILURE,
    })
}

/// `[SIMD-2]`, `[SIMD-5]` — every `@simd` loop's verdict, one line each for
/// `--emit-optimization-report` in source order; an `@simd(assert)` loop not
/// in vectorisable form is `E4020`, naming the first clause it fails and, for
/// a check that remains, the check's reason (`[EFF-11]`). A loop whose
/// counter no counted loop has (or a body copied for each instance, which
/// gives one loop several verdicts: any failing one counts) is said once.
fn simd_verdicts(bodies: &[ember_mir::Body], map: &SourceMap, sink: &mut Sink) -> Vec<String> {
    let mut loops: std::collections::BTreeMap<(ember_span::FileId, u32), (Span, bool, Result<(), ember_mir::NotVectorisable>)> = std::collections::BTreeMap::new();
    for body in bodies {
        for note in &body.loop_notes.simd {
            let verdict = note.verdict.clone().unwrap_or_else(|| {
                Err(ember_mir::NotVectorisable {
                    clause: 1,
                    what: "its trip count is not known before entry".to_string(),
                    check_reason: None,
                    at: None,
                })
            });
            let entry = loops.entry((note.span.file, note.span.start)).or_insert((note.span, note.assert, Ok(())));
            if entry.2.is_ok() {
                entry.2 = verdict;
            }
        }
    }
    let mut lines = Vec::new();
    for (span, assert, verdict) in loops.into_values() {
        match verdict {
            Ok(()) => lines.push(format!("{}: `@simd` loop is in vectorisable form, so its checks are grouped and the C compiler may vectorise it [SIMD-5]", map.location(span))),
            Err(why) => {
                let reason = why.check_reason.map(|reason| format!(" (the check's reason: {reason})")).unwrap_or_default();
                lines.push(format!("{}: `@simd` loop is not in vectorisable form: {}{reason} [SIMD-5]", map.location(span), why.what));
                if assert {
                    let mut diagnostic = Diagnostic::error(codes::E4020, span, format!("`@simd(assert)` loop did not vectorise: {}", why.what));
                    if let Some(at) = why.at.filter(|at| !at.is_dummy()) {
                        diagnostic = diagnostic.secondary(at, "here");
                    }
                    if let Some(reason) = why.check_reason {
                        diagnostic = diagnostic.note(format!("the check that remains has the reason `{reason}` [EFF-11]"));
                    }
                    sink.emit(diagnostic.note(format!("this is clause {} of vectorisable form [SIMD-5]", why.clause)));
                }
            }
        }
    }
    lines
}

/// `[CT-1]` — how many times a program is checked at most: each pass answers
/// what the one before asked.
const COMPTIME_PASSES: usize = 8;

/// `[CT-1]`, `[CT-3]` — what the compile-time evaluator's program is built
/// with: the build's C compiler and profile, a directory of its own under the
/// build's, and the manifest's `[comptime]` limits.
struct ComptimeBuild {
    toolchain: Toolchain,
    profile: Profile,
    for_msvc: bool,
    runtime: PathBuf,
    dir: PathBuf,
    root: PathBuf,
    limits: comptime::Limits,
}

impl ComptimeBuild {
    fn build(&self) -> comptime::Build<'_> {
        comptime::Build {
            toolchain: &self.toolchain,
            profile: self.profile,
            for_msvc: self.for_msvc,
            runtime: &self.runtime,
            c_dir: self.dir.join("c"),
            obj_dir: self.dir.join("obj"),
            bin_dir: self.dir.join("bin"),
            cache_dir: self.dir.join("cache"),
            root: self.root.clone(),
            limits: self.limits,
        }
    }
}

fn comptime_build(input: &Path, options: &Options) -> Result<ComptimeBuild, String> {
    let requested = options
        .cc
        .clone()
        .or_else(|| std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty()));
    let toolchain = Toolchain::detect(requested.as_deref()).map_err(|e| e.to_string())?;
    let target_dir = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target"));
    let layout = Layout::new(&target_dir, options.profile).map_err(|e| e.to_string())?;
    // One directory per source file, so builds of two programs side by side
    // never share the evaluator's files.
    let stem = input.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.to_string_lossy().bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    let dir = layout.c.parent().unwrap_or(&layout.c).join("comptime").join(format!("{stem}-{hash:016x}"));
    let mut limits = comptime::Limits::default();
    if let Some(manifest) = nearest_manifest_text(input)? {
        // `[MAN-8]` — `max_steps`, and `max_heap_mb` in megabytes (D-621).
        if let Some(steps) = manifest_number(&manifest, "comptime", "max_steps") {
            limits.steps = steps;
        }
        if let Some(heap) = manifest_number(&manifest, "comptime", "max_heap_mb").and_then(|mb| mb.checked_mul(1 << 20)) {
            limits.heap = heap;
        }
    }
    // `[CT-2]` — the package directory: the manifest's, else the file's own.
    let root = nearest_manifest_path(input)
        .and_then(|manifest| manifest.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| package_source_root(input));
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    Ok(ComptimeBuild { toolchain, profile: options.profile, for_msvc: c_for_msvc(options), runtime: runtime_dir()?, dir, root, limits })
}

/// A whole number in the manifest, written bare or as a string, with an
/// optional `KB`, `MB` or `GB` (`[comptime] heap = "512MB"`).
fn manifest_number(manifest: &str, wanted_section: &str, wanted_key: &str) -> Option<u64> {
    let mut section = "";
    for line in manifest.lines() {
        let line = line.split_once('#').map_or(line, |(before, _)| before).trim();
        if let Some(name) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            section = name.trim();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        if section != wanted_section || key.trim() != wanted_key {
            continue;
        }
        let value = value.trim().trim_matches('"').replace('_', "");
        let (digits, scale) = match value.strip_suffix("GB").or_else(|| value.strip_suffix("MB")).or_else(|| value.strip_suffix("KB")) {
            Some(digits) if value.ends_with("GB") => (digits.to_string(), 1u64 << 30),
            Some(digits) if value.ends_with("MB") => (digits.to_string(), 1u64 << 20),
            Some(digits) => (digits.to_string(), 1u64 << 10),
            None => (value.clone(), 1),
        };
        return digits.trim().parse::<u64>().ok()?.checked_mul(scale);
    }
    None
}

/// D-464 — what stopped a program whose status is not an exit code: a Windows
/// exception or a Unix signal, named. A panic ends through `abort()` (status
/// 3 on Windows, `SIGABRT` on Unix) after printing its own message, so it
/// gets none here.
fn abnormal_end(status: &std::process::ExitStatus) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        let signal = status.signal()?;
        let what = match signal {
            6 => return None,
            11 => "a segmentation fault (a stack overflow, or a bad memory access)",
            7 => "a bus error",
            4 => "an illegal instruction",
            8 => "an arithmetic fault",
            9 => "a kill",
            _ => "a signal",
        };
        Some(format!("the program was stopped by {what} (signal {signal})"))
    }
    #[cfg(windows)]
    {
        let code = status.code()?;
        if (0..=255).contains(&code) {
            return None;
        }
        let code = code as u32;
        let what = match code {
            0xC000_00FD => "a stack overflow",
            0xC000_0005 => "an access violation",
            0xC000_0409 => "a fast fail",
            0xC000_001D => "an illegal instruction",
            0xC000_0094 => "an integer division by zero",
            _ => "an exception",
        };
        Some(format!("the program was stopped by {what} (0x{code:08X})"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = status;
        None
    }
}

/// `[MAN-3]` — read lint settings from the nearest package and reject every
/// lint name outside the registry. The manifest is loaded into the source map,
/// so an `E9010` points at the offending manifest entry rather than at an
/// unrelated invoking source file.
#[derive(Copy, Clone, Default)]
struct ManifestLintSettings {
    return_intersection: bool,
    long_term_access_across_dynamic_call: bool,
    /// `[EFF-19]` — `[realtime] contracts`, when the manifest declares them.
    realtime: Option<ember_hir::Effects>,
    /// `[MAN-7]` — `[build] reload`, when written and valid, with where.
    reload: Option<(ReloadMode, Span)>,
    /// `[HR-29]` — `[build] runtime = "static"`, when written, with where.
    static_runtime: Option<Span>,
    /// `[MONO-3]` — `[build] max_instantiations`, when written, with where.
    max_instantiations: Option<(usize, Span)>,
    /// `[LNT-6]` — the levels `[lints]` gives `ember lint`'s lints, and `large_copy`'s threshold.
    perf: ember_analysis::PerfLints,
}

/// `[MAN-7]`, `[HR-25]` — what a package's `[build] reload` asks for. `OptIn` makes reloadable only
/// what `@reloadable` names; `All` everything but what `@noreload` names; `Bodies` is `All` admitting
/// only body changes (`[HR-19]`).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum ReloadMode {
    None,
    OptIn,
    All,
    Bodies,
}

impl ReloadMode {
    const VALUES: [(&'static str, ReloadMode); 4] =
        [("opt-in", ReloadMode::OptIn), ("all", ReloadMode::All), ("bodies", ReloadMode::Bodies), ("none", ReloadMode::None)];

    fn parse(value: &str) -> Option<ReloadMode> {
        ReloadMode::VALUES.iter().find(|(name, _)| *name == value).map(|(_, mode)| *mode)
    }
}

fn manifest_lint_settings(
    start: &Path,
    map: &mut SourceMap,
    sink: &mut Sink,
) -> ManifestLintSettings {
    let mut directory = Some(start);
    while let Some(candidate) = directory {
        let path = candidate.join(ember_branding::MANIFEST);
        if path.is_file() {
            let Ok(manifest_file) = map.load(&path) else {
                return ManifestLintSettings::default();
            };
            let text = map.file(manifest_file).text.clone();
            let mut section = "";
            let mut settings = ManifestLintSettings::default();
            let mut line_start = 0usize;
            for raw_line in text.lines() {
                let line = raw_line.split('#').next().unwrap_or("").trim();
                let location = Span::new(manifest_file, line_start as u32, (line_start + raw_line.len()) as u32);
                if line.starts_with('[') && line.ends_with(']') {
                    section = line[1..line.len() - 1].trim();
                    // `[MAN-1]`, `[MAN-8]` — a section the manifest does not have.
                    if manifest_section_keys(section).is_none() {
                        sink.emit(
                            Diagnostic::error(codes::E9001, location, format!("unknown manifest section `[{section}]`"))
                                .help(format!("the manifest's sections are {}", MANIFEST_SECTIONS.join(", "))),
                        );
                    }
                } else if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim().trim_matches('"');
                    if matches!(key, "exclusivity" | "overflow" | "bounds_checks")
                        || key == "gpu.validate"
                        || (section == "gpu" && key == "validate")
                    {
                        let removed = if key == "validate" { "gpu.validate" } else { key };
                        sink.emit(Diagnostic::error(
                            codes::E9001,
                            location,
                            format!("removed manifest setting `{removed}` cannot disable a safety check"),
                        ));
                    } else if let Some(Some(accepted)) = manifest_section_keys(section)
                        && !accepted.contains(&key)
                    {
                        // `[MAN-1]` — the key, and the keys its section accepts.
                        let section = if section.is_empty() {
                            "the manifest's top level".to_string()
                        } else {
                            format!("`[{section}]`")
                        };
                        let accepted: Vec<String> = accepted.iter().map(|key| format!("`{key}`")).collect();
                        let help = if accepted.is_empty() {
                            format!("{section} has no keys: each key belongs to one of the sections {}", MANIFEST_SECTIONS.join(", "))
                        } else {
                            format!("{section} accepts {}", accepted.join(", "))
                        };
                        sink.emit(
                            Diagnostic::error(codes::E9001, location, format!("unknown key `{key}` in {section}")).help(help),
                        );
                    } else if section == "package" && key == "version"
                        && packages::Version::parse(value.trim().trim_matches('"')).is_none()
                    {
                        // `[VER-5]` — package versions are semantic.
                        sink.emit(
                            Diagnostic::error(codes::E9001, location, format!("`{}` is not a semantic version", value.trim()))
                                .help("write `version = \"MAJOR.MINOR.PATCH\"`, such as \"0.1.0\" [VER-5]"),
                        );
                    } else if section == "package" && key == "language" {
                        // `[VER-2]` — the language version a package opts into; before 1.0 there is
                        // one, the current one (`[VER-8]`).
                        let written = value.trim().trim_matches('"');
                        if written != ember_parser::LANGUAGE_VERSION {
                            sink.emit(
                                Diagnostic::error(codes::E9001, location, format!("`language = \"{written}\"` is not the current language"))
                                    .note(format!("before 1.0 there is one language, the current one ({}) [VER-8]; a package opts into a later major version with this key [VER-2]", ember_parser::LANGUAGE_VERSION))
                                    .help(format!("write `language = \"{}\"`", ember_parser::LANGUAGE_VERSION)),
                            );
                        }
                    } else if section == "realtime" && key == "contracts" {
                        settings.realtime = Some(realtime_contracts(value, location, sink));
                    } else if section == "build" && key == "reload" {
                        // `[MAN-7]` — one of four words.
                        let written = value.trim().trim_matches('"');
                        match ReloadMode::parse(written) {
                            Some(mode) => settings.reload = Some((mode, location)),
                            None => sink.emit(
                                Diagnostic::error(codes::E9031, location, format!("`{written}` is not a `reload` value"))
                                    .help("`[build] reload` is \"opt-in\", \"all\", \"bodies\" or \"none\" [MAN-7]"),
                            ),
                        }
                    } else if section == "build" && key == "runtime" && value.trim().trim_matches('"') == "static" {
                        settings.static_runtime = Some(location);
                    } else if section == "build" && key == "max_instantiations" {
                        // `[MONO-3]` — a per-generic ceiling, a positive whole number.
                        match value.trim().parse::<usize>() {
                            Ok(ceiling) if ceiling > 0 => settings.max_instantiations = Some((ceiling, location)),
                            _ => sink.emit(
                                Diagnostic::error(codes::E9034, location, format!("`max_instantiations = {}` is not a positive whole number", value.trim()))
                                    .help("write the most instances one generic may have, such as `max_instantiations = 64` [MONO-3]"),
                            ),
                        }
                    } else if section == "lints" {
                        if !manifest_lint_is_known(key) {
                            sink.emit(Diagnostic::error(
                                codes::E9010,
                                location,
                                format!("unknown lint `{key}` in `[lints]`"),
                            ));
                        } else if key.eq_ignore_ascii_case("l3014") {
                            settings.return_intersection |=
                                matches!(value.trim().trim_matches('"'), "warn" | "deny");
                        } else if key.eq_ignore_ascii_case("l3013") {
                            settings.long_term_access_across_dynamic_call |=
                                matches!(value.trim().trim_matches('"'), "warn" | "deny");
                        } else if let Some(code) = perf_lint_code(key) {
                            perf_lint_setting(code, key, value, location, &mut settings.perf, sink);
                        }
                    }
                }
                line_start += raw_line.len() + 1;
            }
            return settings;
        }
        directory = candidate.parent();
    }
    ManifestLintSettings::default()
}

/// Annex B — the build's reload mode: the manifest's `[build] reload`, or `all` when `--reload` or
/// `--hot` asks for reload and the manifest writes none. `[PRF-2]`, `[MAN-7]`: reload is a build mode
/// of `debug` and `development`; a `release` build ignores it with a note (ODR-118, which retired
/// `E9033`). `[HR-29]`: a reloadable package
/// shares the one runtime of its process, so a manifest asking for a static runtime is `E9036`.
/// `[ABI-1]`, `[FFI-28]` (ODR-131) — the marker a host looks up before calling into a module: its
/// name carries the module protocol's version, `EMBER_MODULE_PROTOCOL` in the runtime's header.
const MODULE_PROTOCOL_MARKER: &str = "module_protocol_v1";

fn reload_mode(settings: &ManifestLintSettings, options: &Options, sink: &mut Sink) -> Result<ReloadMode, String> {
    let (written, location) = settings.reload.unwrap_or((ReloadMode::None, Span::DUMMY));
    let asked = options.reload || options.hot;
    if asked && settings.reload.is_some_and(|(mode, _)| mode == ReloadMode::None) {
        return Err("this package is not reloadable: its manifest says `[build] reload = \"none\"`".to_string());
    }
    let mode = if written == ReloadMode::None && asked { ReloadMode::All } else { written };
    if mode == ReloadMode::None {
        return Ok(mode);
    }
    if options.profile == Profile::Release {
        let _ = location;
        eprintln!("note: hot reload is off in release builds [MAN-7]");
        return Ok(ReloadMode::None);
    }
    if let Some(at) = settings.static_runtime {
        sink.emit(
            Diagnostic::error(codes::E9036, at, "a reloadable package cannot link its own copy of the runtime")
                .note("with reload on, one shared runtime serves every image in the process [HR-29]")
                .help("write `[build] runtime = \"shared\"`, or leave `runtime` out"),
        );
    }
    Ok(mode)
}

/// `[EFF-19]` — `[realtime] contracts = ["noalloc", "nopanic(explicit)", …]`: the contracts
/// `@realtime` stands for in this package. A name that is no contract is `E9001`.
fn realtime_contracts(value: &str, location: Span, sink: &mut Sink) -> ember_hir::Effects {
    let mut set = ember_hir::Effects::NONE;
    let value = value.trim();
    let Some(items) = value.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) else {
        sink.emit(
            Diagnostic::error(codes::E9001, location, "`[realtime] contracts` is a list of contract names")
                .help("write `contracts = [\"noalloc\", \"nolock\", \"noblock\", \"nopanic(explicit)\"]`"),
        );
        return ember_hir::Effects::REALTIME_DEFAULT;
    };
    for item in items.split(',').map(|item| item.trim().trim_matches('"')).filter(|item| !item.is_empty()) {
        match ember_hir::Effects::CONTRACTS.iter().find(|(_, name)| *name == item) {
            Some((effect, _)) => set |= *effect,
            None => {
                let names: Vec<String> = ember_hir::Effects::CONTRACTS.iter().map(|(_, name)| format!("`{name}`")).collect();
                sink.emit(
                    Diagnostic::error(codes::E9001, location, format!("`{item}` is not a contract `@realtime` can stand for"))
                        .help(format!("the contracts are {}", names.join(", "))),
                );
            }
        }
    }
    set
}

/// `[MAN-3]` accepts every registered `L` code by its rendered spelling, plus
/// the descriptive names the manifest examples establish. Configuration for a
/// lint can therefore be validated before that lint has a producer.
fn manifest_lint_is_known(key: &str) -> bool {
    ember_diag::codes::lookup(key)
        .is_some_and(|code| code.kind == ember_diag::codes::CodeKind::Lint)
        || matches!(key, "unused" | "potential_cycle" | "large_copy")
}

/// `[LNT-6]` — the `ember lint` lint a `[lints]` key names: its code, or `large_copy` (`L2002`).
fn perf_lint_code(key: &str) -> Option<ember_diag::codes::Code> {
    if key == "large_copy" {
        return Some(codes::L2002);
    }
    ember_diag::codes::lookup(key).filter(|code| ember_analysis::PerfLints::CODES.contains(code))
}

/// `[MAN-3]`, `[LNT-6]` — one `[lints]` entry for an `ember lint` lint: a level (`"allow"`, `"warn"`,
/// `"error"`), or a table with `level` and the lint's parameters (`large_copy = { threshold = 64 }`).
fn perf_lint_setting(
    code: ember_diag::codes::Code,
    key: &str,
    value: &str,
    location: Span,
    perf: &mut ember_analysis::PerfLints,
    sink: &mut Sink,
) {
    let level = |text: &str, sink: &mut Sink| {
        let text = text.trim().trim_matches('"');
        let level = ember_analysis::LintLevel::parse(text);
        if level.is_none() {
            sink.emit(
                Diagnostic::error(codes::E9001, location, format!("`{text}` is not a lint level"))
                    .help("a lint's level is \"allow\", \"warn\" or \"error\" [MAN-3]"),
            );
        }
        level
    };
    let value = value.trim();
    let Some(table) = value.strip_prefix('{').and_then(|rest| rest.strip_suffix('}')) else {
        if let Some(level) = level(value, sink) {
            perf.set(code, level);
        }
        return;
    };
    for entry in table.split(',').map(str::trim).filter(|entry| !entry.is_empty()) {
        let (name, written) = entry.split_once('=').map_or((entry, ""), |(name, written)| (name.trim(), written.trim()));
        match name {
            "level" => {
                if let Some(level) = level(written, sink) {
                    perf.set(code, level);
                }
            }
            "threshold" if code == codes::L2002 => match written.parse::<u64>() {
                Ok(bytes) => perf.large_copy_threshold = bytes,
                Err(_) => sink.emit(
                    Diagnostic::error(codes::E9001, location, format!("`threshold = {written}` is not a number of bytes"))
                        .help("write `large_copy = { threshold = 256 }` [LNT-6]"),
                ),
            },
            _ => sink.emit(
                Diagnostic::error(codes::E9001, location, format!("`{key}` has no parameter `{name}`"))
                    .help(if code == codes::L2002 { "its table takes `level` and `threshold` [LNT-6]" } else { "its table takes `level` [MAN-3]" }),
            ),
        }
    }
}

/// `[LNT-6]` (`L5001`) — an `unsafe extern` function declaration of the program's own that takes or
/// returns a raw pointer and states no contract (`@ffi(…)`): every call needs `unsafe`, and nothing
/// is known of what C does with the pointer.
fn lint_foreign_contracts(modules: &[ember_typeck::LoadedModule], perf: &ember_analysis::PerfLints, sink: &mut Sink) {
    let pointer = |ty: &ember_ast::TypeExpr| matches!(ty.kind, ember_ast::TypeKind::Ptr { .. });
    for loaded in modules {
        if loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE) {
            continue;
        }
        for item in &loaded.module.items {
            let ember_ast::ItemKind::ExternBlock(block) = &item.kind else { continue };
            for inner in &block.items {
                let ember_ast::ItemKind::Fn(decl) = &inner.kind else { continue };
                if decl.is_safe || inner.attrs.iter().any(|attr| attr.path.first().is_some_and(|name| name.name.as_str() == "ffi")) {
                    continue;
                }
                let takes = decl.params.iter().any(|param| matches!(&param.kind, ember_ast::ParamKind::Named { ty, .. } if pointer(ty)));
                if !takes && !decl.ret.as_ref().is_some_and(pointer) {
                    continue;
                }
                perf.emit(
                    sink,
                    Diagnostic::lint(codes::L5001, decl.name.span, format!("`{}` is an `unsafe extern` declaration with no contract", decl.name.name))
                        .primary_label("each call needs `unsafe`, and nothing is known of its pointers")
                        .help("state what C does with each pointer in `@ffi(param(name, …), result(…))`, and mark it `safe` when the contract makes every call safe [FFI-11]"),
                );
            }
        }
    }
}

/// `[MAN-8]` — the manifest's sections, for `E9001`'s help.
const MANIFEST_SECTIONS: &[&str] = &[
    "`[package]`", "`[dependencies]`", "`[build]`", "`[profiles.<name>]`", "`[c]`", "`[link]`", "`[jobs]`",
    "`[realtime]`", "`[comptime]`", "`[lints]`", "`[cpp.<project>]`", "`[ffi]`",
];

/// `[MAN-1]`, `[MAN-8]` — the keys a manifest section accepts: `None` for a
/// section the manifest does not have, `Some(None)` for one whose keys are
/// names (`[dependencies]`' packages, and `[lints]`' lints, which `[MAN-3]`
/// checks). The top level (`""`) has none.
fn manifest_section_keys(section: &str) -> Option<Option<&'static [&'static str]>> {
    Some(Some(match section {
        "" => &[],
        "package" => &["name", "version", "language", "kind", "layers", "edition_lints"],
        "build" => &["entry", "target", "backend", "c_compiler", "runtime", "max_instantiations", "reload"],
        // `[BLD-FFI-1]`: the C compiler and its flags.
        "c" => &["compiler", "flags"],
        "link" => &["libs"],
        "jobs" => &["workers"],
        "realtime" => &["contracts"],
        "comptime" => &["max_steps", "max_heap_mb"],
        // Annex C's `[ffi] evidence`.
        "ffi" => &["evidence"],
        "dependencies" | "lints" => return Some(None),
        // `[PRF-3]` — the profile table's keys.
        _ if section.starts_with("profiles.") => &[
            "inherits", "opt", "debug_info", "strip", "backtrace", "debug_assert", "leak_report", "lock_order",
            "sanitizers", "lto",
        ],
        // `[FFI-50]`.
        _ if section.starts_with("cpp.") => &["compiler", "standard", "defines", "include_paths", "flags", "cmake"],
        // A dependency written as a table: `{ path }` or `{ git, rev }`.
        _ if section.starts_with("dependencies.") => &["path", "git", "rev"],
        _ => return None,
    }))
}

/// Where `ember_rt`'s sources live. Found relative to the compiler executable
/// during development; an installed toolchain will carry them alongside the
/// binary.
fn runtime_dir() -> Result<PathBuf, String> {
    let from_env = std::env::var_os("EMBER_RUNTIME_DIR").map(PathBuf::from);
    if let Some(dir) = from_env {
        if dir
            .join(format!("include/{}", ember_branding::runtime_header()))
            .is_file()
        {
            return Ok(dir);
        }
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut dir = exe.parent().map(Path::to_path_buf);
    while let Some(candidate) = dir {
        let runtime = candidate.join(format!("runtime/{}_rt", ember_branding::SYMBOL_PREFIX));
        if runtime
            .join(format!("include/{}", ember_branding::runtime_header()))
            .is_file()
        {
            return Ok(runtime);
        }
        dir = candidate.parent().map(Path::to_path_buf);
    }
    Err(format!(
        "cannot find the {}_rt sources; set EMBER_RUNTIME_DIR",
        ember_branding::SYMBOL_PREFIX
    ))
}

fn runtime_artifact_dir(lib: &Path) -> PathBuf {
    lib.join(format!("{}_runtime", ember_branding::SYMBOL_PREFIX))
}

/// Place the public runtime ABI header beside library artifacts under a
/// branded subdirectory. A package named `ember_rt` can then have its own
/// `<package>.h` without a self-include or archive-name collision (D-366).
fn copy_runtime_header(lib: &Path) -> Result<PathBuf, String> {
    let name = ember_branding::runtime_header();
    let source = runtime_dir()?.join("include").join(&name);
    let directory = runtime_artifact_dir(lib);
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let destination = directory.join(name);
    std::fs::copy(source, &destination).map_err(|error| error.to_string())?;
    Ok(destination)
}

/// `[TYP-8]` (0.9.9) — "Integer overflow panics in every profile", and
/// `[PRF-1]`: a profile never changes what a program means. 0.8's `release`
/// and `shipping` wrapped; wrapping is now only what `@overflow(wrap)` or the
/// `wrapping_*` methods ask for.
fn overflow_policy(profile: Profile) -> ember_types::OverflowPolicy {
    match profile {
        Profile::Debug | Profile::Development | Profile::Release => ember_types::OverflowPolicy::Panic,
    }
}

fn report(sink: &Sink, map: &SourceMap, options: &Options) {
    report_timings(options);
    if sink.diagnostics().is_empty() {
        return;
    }
    if options.json {
        print!("{}", sink.render_json(map));
    } else {
        eprint!("{}", sink.render(map));
    }
}

/// `[BLD-9]` — each stage's time since the one before, with the module it
/// ran on (`all modules` for a stage that runs over the program at once),
/// in the order they ran. Off (`None`) unless `--timings` started it.
type Timings = Option<(std::time::Instant, std::time::Instant, Vec<(String, String, std::time::Duration)>)>;
static TIMINGS: std::sync::Mutex<Timings> = std::sync::Mutex::new(None);

fn start_timings() {
    let now = std::time::Instant::now();
    *TIMINGS.lock().unwrap_or_else(|e| e.into_inner()) = Some((now, now, Vec::new()));
}

fn mark(stage: &str, module: &str) {
    if let Some((_, last, stages)) = TIMINGS.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        let now = std::time::Instant::now();
        stages.push((stage.to_string(), module.to_string(), now - *last));
        *last = now;
    }
}

/// Prints the timings once, on standard error (as one JSON line with
/// `--timings=json`), so they never mix with `--emit c` on standard output.
fn report_timings(options: &Options) {
    let Some(json) = options.timings else { return };
    let Some((start, last, stages)) = TIMINGS.lock().unwrap_or_else(|e| e.into_inner()).take() else { return };
    let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
    if json {
        let stages: Vec<serde_json::Value> = stages
            .iter()
            .map(|(stage, module, took)| serde_json::json!({ "stage": stage, "module": module, "ms": ms(*took) }))
            .collect();
        eprintln!("{}", serde_json::json!({ "timings": stages, "total_ms": ms(last - start) }));
        return;
    }
    eprintln!("timings (ms):");
    for (stage, module, took) in &stages {
        eprintln!("  {stage:<22}{:>10.2}  {module}", ms(*took));
    }
    eprintln!("  {:<22}{:>10.2}", "total", ms(last - start));
}

/// `[UNS-8]` — `W3012` for each `unsafe:` block or `unsafe fn` with no
/// safety note (`[LEX-23]`): a `SAFETY` comment alone on the line before it,
/// or at the end of its first line.
fn check_safety_notes(module: &ember_ast::Module, comments: &[ember_lexer::Comment], map: &SourceMap, sink: &mut Sink) {
    if module.unsafe_sites.is_empty() {
        return;
    }
    let line = |span: Span| map.file(span.file).line_col(span.start).line;
    let noted: std::collections::HashSet<_> =
        comments.iter().filter(|comment| comment.safety.is_some()).map(|comment| (line(comment.span), comment.own_line)).collect();
    for &site in &module.unsafe_sites {
        let at = line(site);
        if noted.contains(&(at, false)) || (at > 1 && noted.contains(&(at - 1, true))) {
            continue;
        }
        sink.emit(
            Diagnostic::warning(codes::W3012, site, "this `unsafe` has no safety note")
                .primary_label("nothing here says why it is sound")
                .help("write `# SAFETY: <why it is sound>` on the line before it or at the end of this line; `SAFETY(ffi)`, `(layout)`, `(aliasing)`, `(intrinsic)`, `(performance)` or `(uninit)` names its category"),
        );
    }
}

/// Why a source file could not be loaded: `E0001` when it is not UTF-8
/// (`[LEX-1]`), `E1010` when it cannot be read.
fn load_diagnostic(error: &LoadError, span: Span) -> Diagnostic {
    match error {
        LoadError::NotUtf8(path) => Diagnostic::error(codes::E0001, span, format!("`{}` is not valid UTF-8", path.display()))
            .help("source files are UTF-8; save it in that encoding"),
        LoadError::Io(path, _) => Diagnostic::error(codes::E1010, span, format!("cannot read `{}`: {error}", path.display())),
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

#[cfg(test)]
mod tests {
    use super::abnormal_end;

    /// D-464 — a crash is named, and a panic's own end (it has printed its
    /// message) and an ordinary exit code are not.
    #[test]
    fn a_crash_is_named_and_a_panic_or_an_exit_is_not() {
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            let crash = abnormal_end(&std::process::ExitStatus::from_raw(0xC000_00FD)).expect("a crash is named");
            assert!(crash.contains("stack overflow") && crash.contains("0xC00000FD"), "{crash}");
            assert_eq!(abnormal_end(&std::process::ExitStatus::from_raw(3)), None);
            assert_eq!(abnormal_end(&std::process::ExitStatus::from_raw(0)), None);
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            // A raw wait status: the low seven bits are the signal.
            let crash = abnormal_end(&std::process::ExitStatus::from_raw(11)).expect("a crash is named");
            assert!(crash.contains("segmentation fault") && crash.contains("signal 11"), "{crash}");
            assert_eq!(abnormal_end(&std::process::ExitStatus::from_raw(6)), None);
            assert_eq!(abnormal_end(&std::process::ExitStatus::from_raw(3 << 8)), None);
        }
    }
}
