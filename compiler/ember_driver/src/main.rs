//! The `ember` command (Part XIX §1).
//!
//! Phase 0 implements `build`, `run`, `check`, `explain` and the `--emit`
//! stage dumps. The rest of the CLI surface arrives with the phases that give
//! each command something to do.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ember_build::interface::{
    CallableInterfaceRecord, ModuleInterfaceArtifact, ModuleInterfaceInput, PreparedInterfaceCache,
    build_artifacts, cache_directory, prepare_interface_cache, round_trip_artifacts,
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
                | ember_mir::StmtKind::Nop => {}
            }
        }
        match &block.terminator {
            ember_mir::Terminator::SwitchInt { discr, .. } => {
                collect_operand_function_symbols(discr, out);
            }
            ember_mir::Terminator::Call { func, args, .. } => {
                match func {
                    ember_mir::FuncRef::Direct { symbol } => {
                        out.insert(symbol.clone());
                    }
                    ember_mir::FuncRef::Indirect(operand) => {
                        collect_operand_function_symbols(operand, out);
                    }
                    ember_mir::FuncRef::Builtin { .. } => {}
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
                    | ember_mir::AssertKind::ShiftTooLarge => {}
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
    map: &SourceMap,
    bodies: &mut [ember_mir::Body],
    types: &TypeTable,
    options: &Options,
) -> Result<PreparedInterfaceCache, String> {
    ember_analysis::install_callable_regions_all(bodies, types);

    let inputs = module_interface_inputs(modules, map, bodies, options)?;
    let fresh = build_artifacts(&inputs, env!("CARGO_PKG_VERSION")).map_err(|error| {
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
    install_callable_metadata_from_artifacts(bodies, prepared.artifacts())?;
    Ok(prepared)
}

fn module_interface_inputs(
    modules: &[ember_typeck::LoadedModule],
    map: &SourceMap,
    bodies: &[ember_mir::Body],
    options: &Options,
) -> Result<Vec<ModuleInterfaceInput>, String> {
    use std::collections::{BTreeMap, BTreeSet};

    let module_names: BTreeMap<ember_span::FileId, String> = modules
        .iter()
        .map(|loaded| (loaded.module.span.file, module_identity(&loaded.path)))
        .collect();
    let known_modules: BTreeSet<String> = module_names.values().cloned().collect();
    let mut records_by_file: BTreeMap<ember_span::FileId, Vec<CallableInterfaceRecord>> =
        BTreeMap::new();
    for body in bodies {
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
        if !module_names.contains_key(&body.span.file) {
            return Err(format!(
                "internal compiler error: [BLD-2] callable `{}` has no source module for its interface artifact",
                body.symbol
            ));
        }
        records_by_file
            .entry(body.span.file)
            .or_default()
            .push(CallableInterfaceRecord {
                symbol: body.symbol.clone(),
                metadata: metadata.clone(),
            });
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
                .unwrap_or_default(),
        });
    }
    if let Some((file, records)) = records_by_file.into_iter().next() {
        return Err(format!(
            "internal compiler error: [BLD-2] {} callable interface record(s) have no loaded module for source file {}",
            records.len(),
            file.0
        ));
    }
    Ok(inputs)
}

fn install_callable_metadata_from_artifacts(
    bodies: &mut [ember_mir::Body],
    artifacts: &[ModuleInterfaceArtifact],
) -> Result<(), String> {
    use std::collections::BTreeMap;

    let mut records: BTreeMap<String, ember_mir::CallableRegionMetadata> = BTreeMap::new();
    for artifact in artifacts {
        for (symbol, metadata) in &artifact.callables {
            if records.insert(symbol.clone(), metadata.clone()).is_some() {
                return Err(format!(
                    "internal compiler error: [LT-40] callable `{symbol}` is present in more than one interface artifact"
                ));
            }
        }
    }
    for body in bodies {
        body.callable_regions = Some(records.remove(&body.symbol).ok_or_else(|| {
            format!(
                "internal compiler error: [LT-40] interface artifact omits callable `{}`",
                body.symbol
            )
        })?);
    }
    if let Some((symbol, _)) = records.into_iter().next() {
        return Err(format!(
            "internal compiler error: [LT-40] interface artifact contains stale callable `{symbol}`"
        ));
    }
    Ok(())
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
    let modules = load_modules(module, &root_dir, &mut map, &mut sink);
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // Check.
    let (mut types, common) = TypeTable::new();
    // [TYP-8] -- the profile chooses the default overflow policy.
    let program = ember_typeck::check(
        &modules,
        &mut types,
        &common,
        &mut sink,
        overflow_policy(options.profile),
    );
    if options.emit.as_deref() == Some("hir") {
        print!("{}", ember_hir::dump(&program, &types));
        return Ok(finish(&sink, &map, options));
    }
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
    }

    // Lower. `ember check` runs the MIR analyses too — Part XIX §1 defines it
    // as "type-check + borrow-check without codegen", so it cannot stop here.
    let mut bodies = ember_mir::lower(&program, &types, &common, &map);
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
    let interface_cache =
        prepare_callable_interface_cache(input, &modules, &map, &mut bodies, &types, options)?;
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
    }
    if sink.has_errors() {
        return Ok(finish(&sink, &map, options));
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
