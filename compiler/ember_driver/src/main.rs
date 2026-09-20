//! The `ember` command (Part XIX §1).
//!
//! Phase 0 implements `build`, `run`, `check`, `explain` and the `--emit`
//! stage dumps. The rest of the CLI surface arrives with the phases that give
//! each command something to do.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ember_build::interface::{
    CallableGenericCallableBound, CallableGenericParameter, CallableInterfaceContract,
    CallableInterfaceRecord, CallableParameter, CallableParameterMode, CallableSignature,
    ModuleInterfaceArtifact, ModuleInterfaceInput, PreparedInterfaceCache,
    build_artifacts, cache_directory, compiler_identity, prepare_interface_cache,
    round_trip_artifacts,
};
use ember_build::{Layout, LinkRequest, Profile, Toolchain};
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
    ember inspect --safety [options] <path>
                                      report emitted/elided safety checks

options:
    --profile debug|release|shipping   default: debug
    --emit tokens|ast|hir|mir|c        print an intermediate form and stop
    --syntax-only                      lex and parse only; report E00xx/E01xx
    --backend c                        the only backend in v1
    --cc msvc|clang|gcc                override C compiler detection
    --out-dir <dir>                    default: target/
    --json                             machine-readable diagnostics
    -D warnings                        treat warnings as errors

inspect options:
    --function <name>                  restrict safety output to one function
    --elided-only                      report only statically elided checks
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
fn retain_referenced_standard_bodies(bodies: &mut Vec<ember_mir::Body>) {
    use std::collections::{BTreeMap, BTreeSet};

    let std_prefix = ember_branding::mangled(&format!("{}.", ember_branding::STD_PACKAGE));
    let by_symbol: BTreeMap<String, usize> = bodies
        .iter()
        .enumerate()
        .map(|(index, body)| (body.symbol.clone(), index))
        .collect();
    let mut keep = BTreeSet::new();
    let mut pending = Vec::new();

    for body in bodies.iter() {
        let standard = body.symbol.starts_with(&std_prefix);
        let drop_glue = body.name == "drop" || body.name.ends_with(".drop");
        if (!standard || drop_glue) && keep.insert(body.symbol.clone()) {
            pending.push(body.symbol.clone());
        }
    }

    while let Some(symbol) = pending.pop() {
        let Some(&index) = by_symbol.get(&symbol) else {
            continue;
        };
        let mut referenced = BTreeSet::new();
        collect_body_function_symbols(&bodies[index], &mut referenced);
        for target in referenced {
            if by_symbol.contains_key(&target) && keep.insert(target.clone()) {
                pending.push(target);
            }
        }
    }

    bodies.retain(|body| !body.symbol.starts_with(&std_prefix) || keep.contains(&body.symbol));
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
                | ember_mir::StmtKind::EndAccess { .. }
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
            let code = args
                .get(1)
                .ok_or("`ember explain` needs a code, e.g. E3040")?;
            explain(code)
        }
        "inspect" => inspect(&args[1..]),
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

fn inspect(args: &[String]) -> Result<ExitCode, String> {
    let mut safety = false;
    let mut elided_only = false;
    let mut json = false;
    let mut function = None;
    let mut path = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--safety" => safety = true,
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

    if !safety {
        return Err("`ember inspect` currently requires `--safety`".to_string());
    }
    let path = path.ok_or("`ember inspect --safety` needs a side-table path")?;
    inspect_safety(&path, elided_only, function.as_deref(), json)
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

    if json {
        let report = serde_json::json!({ "schema": schema, "checks": selected });
        println!(
            "{}",
            serde_json::to_string(&report).map_err(|error| error.to_string())?
        );
    } else {
        print_safety_report(&resolved, &selected, elided_only, function);
    }
    Ok(ExitCode::SUCCESS)
}

fn resolve_safety_path(path: &Path) -> PathBuf {
    if path.is_file() {
        return path.to_path_buf();
    }
    let candidates = [
        PathBuf::from("target/debug/inspect"),
        PathBuf::from("target/release/inspect"),
        PathBuf::from("target/shipping/inspect"),
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
        Some("emitted" | "elided") => Ok(()),
        Some(status) => Err(format!("unknown safety side-table status `{status}`")),
        None => unreachable!("status was checked above"),
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
        println!("    {source}  {function}  {mechanism}  reason: {reason}");
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
            args.get(*index)
                .cloned()
                .ok_or_else(|| format!("`{name}` needs a value"))
        };
        match arg {
            "--profile" => {
                options.profile = Profile::from_name(&value(&mut index, arg)?)
                    .ok_or("profile must be debug, release or shipping")?
            }
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
    let mut queue: Vec<(Vec<String>, ember_ast::Module)> = Vec::new();

    // `[MOD-5]` — source-backed prelude interfaces must exist even when the
    // user did not write an import. Loading a module does not make all of its
    // public items visible; type checking separately installs only the names
    // in the normative prelude list. Keep the root last so this LIFO worklist
    // still makes it module zero and preserves the command-line entry point.
    for module in ["core", "collections"] {
        let names = vec![ember_branding::STD_PACKAGE.to_string(), module.to_string()];
        let key = names.join(".");
        let Some(file_path) = resolve_module(root_dir, &names) else {
            continue;
        };
        let file = match map.load(&file_path) {
            Ok(file) => file,
            Err(error) => {
                sink.emit(ember_diag::Diagnostic::error(
                    ember_diag::codes::E1010,
                    ember_span::Span::DUMMY,
                    format!("cannot read `{}`: {error}", file_path.display()),
                ));
                continue;
            }
        };
        let text = map.file(file).text.clone();
        let lexed = ember_lexer::lex(file, &text, sink);
        let parsed = ember_parser::parse(file, &text, lexed.tokens, sink);
        seen.insert(key);
        queue.push((names, parsed));
    }
    queue.push((Vec::new(), root));

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

        loaded.push(ember_typeck::LoadedModule {
            path: path.clone(),
            module,
        });

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
                if names
                    .first()
                    .is_some_and(|f| f == ember_branding::STD_PACKAGE)
                {
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
        };
        if import_visible_contracts
            .insert(declaration.symbol.clone(), contract.clone())
            .is_some()
        {
            return Err(format!(
                "internal compiler error: [BLD-2] import-visible callable `{}` has more than one declaration contract",
                declaration.symbol
            ));
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

    for body in bodies {
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
        let contract = CallableInterfaceContract {
            signature,
            metadata: Some(metadata.clone()),
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
            expected.metadata = contract.metadata.clone();
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

    let implicit_prelude: Vec<String> = ["std.core", "std.collections"]
        .into_iter()
        .filter(|module| known_modules.contains(*module))
        .map(str::to_string)
        .collect();
    let package_config = format!("profile={}", options.profile.name());
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
            .directive
            .as_ref()
            .filter(|directive| directive.name.name.is("language"))
            .map(|directive| directive.value.clone())
            // The adopted normative source is currently 0.8.5. The compiler
            // has no manifest parser yet, so this is the explicit default
            // cache input until package configuration becomes real.
            .unwrap_or_else(|| "0.8.5".to_string());
        inputs.push(ModuleInterfaceInput {
            module,
            source: map.file(loaded.module.span.file).text.clone(),
            language_version,
            package_config: package_config.clone(),
            direct_dependencies: dependencies.into_iter().collect(),
            callables: records_by_file
                .remove(&loaded.module.span.file)
                .unwrap_or_default()
                .into_iter()
                .map(|(symbol, contract)| CallableInterfaceRecord { symbol, contract })
                .collect(),
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

fn install_callable_metadata_from_artifacts(
    bodies: &mut [ember_mir::Body],
    artifacts: &[ModuleInterfaceArtifact],
    expected_contracts: &std::collections::BTreeMap<String, CallableInterfaceContract>,
) -> Result<(), String> {
    use std::collections::BTreeMap;

    let mut records: BTreeMap<String, CallableInterfaceContract> = BTreeMap::new();
    for artifact in artifacts {
        for (symbol, contract) in &artifact.callables {
            if records.insert(symbol.clone(), contract.clone()).is_some() {
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
    let root_dir = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let lint_return_intersection = manifest_enables_l3014(&root_dir);
    let modules = load_modules(module, &root_dir, &mut map, &mut sink);
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // Check.
    let (mut types, common) = TypeTable::new();
    // [TYP-8] -- the profile chooses the default overflow policy.
    let checked = ember_typeck::check(
        &modules,
        &mut types,
        &common,
        &mut sink,
        overflow_policy(options.profile),
        lint_return_intersection,
    );
    let program = &checked.program;
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
    // Part XVIII §4.7 — the NLL borrow checker runs on MIR after drop
    // elaboration, so the drops it sees are the ones that will exist.
    ember_analysis::check_all_with_installed_callable_regions(&bodies, &types, &mut sink);
    if !sink.has_errors() {
        verify_callable_regions_or_panic(&bodies, &types);
    }
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
        ember_mir::verify::verify_interface_upcasts_all(&bodies, &types);
    }
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }
    // `[EXC-3]`/`[EXC-3a]` — remove only the access intervals for which the
    // MIR proof establishes a unique, unescaped class handle. All other
    // intervals remain explicit runtime checks.
    ember_analysis::elide_static_accesses_all(&mut bodies, &types);
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
    if program.main.is_some() {
        retain_referenced_standard_bodies(&mut bodies);
    }
    verify_callable_regions_or_panic(&bodies, &types);
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
    let module_name = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let emitted = ember_codegen_c::emit(verified_mir, &map, &module_name, program.main.is_some());
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
    let target_dir = options
        .out_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("target"));
    let layout = Layout::new(&target_dir, options.profile).map_err(|e| e.to_string())?;
    let c_path = layout.c.join(format!("{module_name}.c"));
    std::fs::write(&c_path, &emitted.c_source).map_err(|e| e.to_string())?;
    let safety_path = layout.inspect.join(format!("{module_name}.safety.json"));
    std::fs::write(&safety_path, &emitted.safety_json).map_err(|e| e.to_string())?;

    let runtime = runtime_dir()?;
    let exe_name = if cfg!(windows) {
        format!("{module_name}.exe")
    } else {
        module_name.clone()
    };
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
        let status = std::process::Command::new(&exe)
            .status()
            .map_err(|e| e.to_string())?;
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

fn manifest_enables_l3014(start: &Path) -> bool {
    let mut directory = Some(start);
    while let Some(candidate) = directory {
        let path = candidate.join(ember_branding::MANIFEST);
        if path.is_file() {
            let Ok(text) = std::fs::read_to_string(path) else { return false };
            let mut in_lints = false;
            for line in text.lines() {
                let line = line.split('#').next().unwrap_or("").trim();
                if line.starts_with('[') && line.ends_with(']') {
                    in_lints = line == "[lints]";
                } else if in_lints
                    && let Some((key, value)) = line.split_once('=')
                    && key.trim().eq_ignore_ascii_case("l3014")
                {
                    return matches!(value.trim().trim_matches('"'), "warn" | "deny");
                }
            }
            return false;
        }
        directory = candidate.parent();
    }
    false
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
