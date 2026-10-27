//! `[CT-1]`–`[CT-7]`, `[TYP-41]` — the compile-time evaluator.
//!
//! The checker makes each expression evaluated while compiling a function of
//! its own and asks for its value (`ember_typeck::ComptimeRequest`). This
//! builds those functions, with everything they call, through the ordinary
//! backend into a program of the evaluator's own
//! (`ember_codegen_c::emit_comptime`), runs it once per request and reads each
//! value back; the checker then checks the program again with the answers.
//! The evaluator's program is the target's own code, built by the target's C
//! compiler, so a value is the target's arithmetic, layout and overflow policy
//! (`[CT-4]`). A function whose check was incomplete (an error, or a value of
//! this kind not known yet) is in that program as a stub that stops with a
//! marker: a request that reaches one waits for a later pass, or for the
//! function's own errors to be reported.
//!
//! Before it runs, a request whose function reaches an effect compile time
//! cannot run (`Io`, `FFI`, `Sync`, `Block`, `Unsafe`, `Nondet`) is `E6010`,
//! naming the call chain (`[CT-2]`). A panic is `E6004` with the panic's
//! message and the compile-time call chain (`[CT-7]`); passing the step or
//! heap limit is `E6001` (`[CT-3]`).

use std::path::{Path, PathBuf};

use ember_build::{LinkRequest, Profile, Toolchain};
use ember_codegen_c::ComptimeRoot;
use ember_diag::{Diagnostic, Sink, codes};
use ember_hir as hir;
use ember_span::SourceMap;
use ember_typeck::{CheckOutput, ComptimeAnswer, ComptimeAnswers, ComptimeRequest};
use ember_types::{CommonTypes, TypeTable};

/// `[CT-3]` — each evaluation's limits: steps (a function entered or a loop
/// turned) and bytes of heap live at once. `[comptime]` in the manifest sets
/// them.
#[derive(Clone, Copy)]
pub struct Limits {
    pub steps: u64,
    pub heap: u64,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits { steps: 100_000_000, heap: 256 * 1024 * 1024 }
    }
}

/// What a request's function stops with when it reaches an incomplete one.
const INCOMPLETE: &str = "ember-comptime-incomplete";

/// How the evaluator's program is built.
pub struct Build<'a> {
    /// `None` for a build that runs no C compiler (`ember run --interp`): the requests run on the
    /// evaluator (`ember_interp::comptime`) instead.
    pub toolchain: Option<&'a Toolchain>,
    pub profile: Profile,
    pub for_msvc: bool,
    pub runtime: &'a Path,
    /// Where the program and its objects go.
    pub c_dir: PathBuf,
    pub obj_dir: PathBuf,
    pub bin_dir: PathBuf,
    /// `[CT-4]` — where each value is kept with its inputs.
    pub cache_dir: PathBuf,
    /// `[CT-2]` — the package directory, the one place `read_file` reads.
    pub root: PathBuf,
    pub limits: Limits,
}

/// One pass's requests, evaluated: each gets an answer in `answers`, and a
/// failure the evaluator can name gets a diagnostic in `reports`, except one
/// that reached a function not complete in this pass, which is left to be
/// asked again.
pub fn evaluate(
    mut checked: CheckOutput,
    types: &mut TypeTable,
    common: &CommonTypes,
    map: &SourceMap,
    build: &Build<'_>,
    answers: &mut ComptimeAnswers,
    reports: &mut Vec<Diagnostic>,
) -> Result<(), String> {
    stub_incomplete(&mut checked.program, &checked.incomplete, common);
    let mut bodies = ember_mir::lower(&checked.program, types, common, map);
    // What no request reaches is a stub too, so a body the check left
    // unfinished in a way it did not see cannot stop the build.
    let roots: Vec<String> = checked.comptime.iter().map(|request| request.symbol.clone()).collect();
    let reached = reachable(&bodies, &roots);
    let unreached: Vec<usize> = bodies
        .iter()
        .enumerate()
        .filter(|(_, body)| !reached.contains(&body.symbol) && !ember_branding::symbol_path_starts_with(&body.symbol, &[ember_branding::STD_PACKAGE]) && !body.is_extern_declaration)
        .map(|(index, _)| index)
        .collect();
    if !unreached.is_empty() {
        let symbols: std::collections::HashSet<String> = unreached.iter().map(|&index| bodies[index].symbol.clone()).collect();
        stub_incomplete(&mut checked.program, &symbols, common);
        let functions = ember_mir::function_index(&checked.program);
        for index in unreached {
            bodies[index] = ember_mir::lower_one(&checked.program.functions[index], &checked.program, &functions, types, common, map);
        }
    }
    // `[IMP-11]` — verified after each pass, as the program's own build is.
    let mut mir = ember_mir::verify::Pipeline::new(bodies, types);
    mir.pass("pruning unused implicit bodies", types, |bodies, types| ember_mir::prune_unused_implicit(bodies, types, &checked.std_instances));
    // What these passes would report, the final pass reports for the program.
    let mut scratch = Sink::new();
    mir.pass("drop elaboration", types, |bodies, types| ember_analysis::elaborate_drops_all(bodies, types, &mut scratch));
    mir.pass("the coroutine transform", types, |bodies, types| ember_analysis::coroutines::transform(bodies, types));
    mir.pass("callable region summaries", types, |bodies, types| ember_analysis::install_callable_regions_all(bodies, types));
    mir.pass("inserting shared accesses", types, |bodies, types| ember_analysis::insert_shared_accesses_all(bodies, types));
    let bodies = mir.into_bodies();

    // `[CT-2]` — what a request reaches that compile time cannot run.
    let mut roots = Vec::new();
    for request in &checked.comptime {
        let Some(index) = bodies.iter().position(|body| body.symbol == request.symbol) else {
            answers.insert(request.key.clone(), ComptimeAnswer::Failed);
            continue;
        };
        if let Some((effect, chain)) = ember_analysis::effects::comptime_forbidden(&bodies, types, index) {
            if request.quiet {
                answers.insert(request.key.clone(), ComptimeAnswer::Failed);
                continue;
            }
            let named = ember_hir::Effects::NAMED.iter().find(|(e, _)| *e == effect).map_or("an effect", |(_, name)| name);
            let operation = chain.last().cloned().unwrap_or_default();
            reports.push(
                Diagnostic::error(codes::E6010, request.span, format!("{} cannot run while compiling: it reaches {operation}", request.what))
                    .primary_label(format!("this carries `{named}`"))
                    .note(format!("the call chain: {}", chain.join(" -> ")))
                    .note("compile time runs what carries only `Alloc`, `Panic` and run-time checks: no `Io`, `FFI`, `Sync`, `Block`, `Unsafe` or `Nondet` [CT-1]"),
            );
            answers.insert(request.key.clone(), ComptimeAnswer::Failed);
            continue;
        }
        roots.push((request, bodies[index].return_ty()));
    }
    if roots.is_empty() {
        return Ok(());
    }

    let verified = ember_mir::verify::for_codegen(&bodies, types).map_err(|violations| {
        format!(
            "the compile-time evaluator's MIR failed verification:\n{}",
            violations.iter().map(|v| format!("  {}: {}", v.body, v.message)).collect::<Vec<_>>().join("\n")
        )
    })?;
    let comptime_roots: Vec<ComptimeRoot> =
        roots.iter().map(|(request, ty)| ComptimeRoot { symbol: request.symbol.clone(), ty: *ty }).collect();
    let emitted = ember_codegen_c::emit_comptime(verified, map, comptime_roots, build.for_msvc);

    // `[CT-4]` — a value is kept with its inputs: the program that computes
    // it (its C, so its source and everything it calls), the limits, and the
    // files and variables it read, each checked again before it is reused.
    let mut program = fnv(emitted.c_source.as_bytes(), 0xcbf29ce484222325);
    for (_, unit) in &emitted.relaxed_units {
        program = fnv(unit.as_bytes(), program);
    }
    program = fnv(format!("{}:{}:{}", build.limits.steps, build.limits.heap, build.root.display()).as_bytes(), program);
    // The evaluator's answers are kept apart from the C program's: each is checked against the
    // other by running both (`[IMP-11]`), which a shared answer would hide.
    if build.toolchain.is_none() {
        program = fnv(b"interp", program);
    }
    let keys: Vec<u64> = (0..roots.len()).map(|index| fnv(index.to_string().as_bytes(), program)).collect();
    let mut misses = Vec::new();
    for (index, (request, _)) in roots.iter().enumerate() {
        match cached(&build.cache_dir, keys[index]) {
            Some(value) => {
                answers.insert(request.key.clone(), ComptimeAnswer::Value(value));
            }
            None => misses.push(index),
        }
    }
    if misses.is_empty() {
        return Ok(());
    }
    let exe = match build.toolchain {
        Some(toolchain) => Some(link(build, toolchain, &emitted)?),
        None => None,
    };
    let program = ember_interp::Program {
        bodies: &bodies,
        types,
        common,
        map,
        debug: build.profile == Profile::Debug,
        backtrace: false,
        leak_check: false,
    };

    for index in misses {
        let request = roots[index].0;
        let (code, ended, stdout, stderr) = match &exe {
            Some(exe) => {
                let output = std::process::Command::new(exe)
                    .arg(index.to_string())
                    .arg(build.limits.steps.to_string())
                    .arg(build.limits.heap.to_string())
                    .arg(&build.root)
                    .output()
                    .map_err(|error| format!("cannot run the compile-time evaluator: {error}"))?;
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                (output.status.code(), output.status.to_string(), stdout, String::from_utf8_lossy(&output.stderr).to_string())
            }
            None => {
                let limits = ember_interp::Limits { steps: build.limits.steps, heap: build.limits.heap };
                let run = ember_interp::comptime::evaluate(&program, &request.symbol, limits, &build.root);
                (Some(run.status), format!("status {}", run.status), run.stdout.trim().to_string(), run.stderr)
            }
        };
        let answer = match code {
            Some(0) => {
                keep(&build.cache_dir, keys[index], &stdout, &stderr);
                ComptimeAnswer::Value(stdout)
            }
            // It needs a function a later pass may complete: asked again then.
            Some(86) if stderr.contains(INCOMPLETE) => continue,
            // `[STA-3]` — a static that cannot be evaluated runs on first access.
            Some(_) if request.quiet => ComptimeAnswer::Failed,
            // `[CT-2]` — `read_file` outside the package directory.
            Some(89) => {
                let path = stderr.lines().find_map(|line| line.strip_prefix("ember-comptime-outside ")).unwrap_or("");
                let mut diagnostic = Diagnostic::error(
                    codes::E6010,
                    request.span,
                    format!("evaluating {} reads `{path}`, which is outside the package directory", request.what),
                )
                .note("`comptime.read_file` reads only inside the package directory, so a dependency's build cannot read the builder's files [CT-2]");
                let frames = chain(&stderr);
                if !frames.is_empty() {
                    diagnostic = diagnostic.note(format!("the compile-time call chain: {}", frames.join(" -> ")));
                }
                reports.push(diagnostic);
                ComptimeAnswer::Failed
            }
            Some(86) => {
                reports.push(panicked(request, &stderr));
                ComptimeAnswer::Failed
            }
            Some(87 | 88) => {
                reports.push(exhausted(request, &stderr, build.limits));
                ComptimeAnswer::Failed
            }
            _ => {
                let mut diagnostic = Diagnostic::error(codes::E6004, request.span, format!("evaluating {} stopped abnormally", request.what))
                    .note(format!("the evaluator ended with {ended}"));
                if let Some(line) = stderr.lines().find(|line| line.starts_with("the evaluator stopped")) {
                    diagnostic = diagnostic.note(line.to_string());
                }
                reports.push(diagnostic.note("a compile-time evaluation runs the program's own code [CT-7]"));
                ComptimeAnswer::Failed
            }
        };
        answers.insert(request.key.clone(), answer);
    }
    Ok(())
}

/// The functions the requests reach, and every class method and `drop`
/// (which dispatch and drops reach without naming them).
fn reachable(bodies: &[ember_mir::Body], roots: &[String]) -> std::collections::BTreeSet<String> {
    let by_symbol: std::collections::HashMap<&str, usize> =
        bodies.iter().enumerate().map(|(index, body)| (body.symbol.as_str(), index)).collect();
    let mut seen: std::collections::BTreeSet<String> = roots.iter().cloned().collect();
    for body in bodies {
        // `[CORO-4]` — a generator's body is what its frame's `next` and
        // `drop` become, after this (`ember_analysis::coroutines`): kept
        // whole, as a `drop` is.
        let generator = body.blocks.iter().any(|block| {
            matches!(&block.terminator, ember_mir::Terminator::Call { func: ember_mir::FuncRef::Direct { symbol, .. }, .. }
                if ember_mir::lower::coroutines::is_start(symbol))
        });
        if body.class_owner.is_some() || body.name == "drop" || body.name.ends_with(".drop") || generator {
            seen.insert(body.symbol.clone());
        }
    }
    let mut pending: Vec<String> = seen.iter().cloned().collect();
    while let Some(symbol) = pending.pop() {
        let Some(&index) = by_symbol.get(symbol.as_str()) else { continue };
        let mut referenced = std::collections::BTreeSet::new();
        crate::collect_body_function_symbols(&bodies[index], &mut referenced);
        for target in referenced {
            if seen.insert(target.clone()) {
                pending.push(target);
            }
        }
    }
    seen
}

/// The program as the evaluator builds it: each incomplete function a stub
/// with its signature, which stops with the `INCOMPLETE` marker.
fn stub_incomplete(program: &mut hir::Program, incomplete: &std::collections::HashSet<String>, common: &CommonTypes) {
    for function in &mut program.functions {
        if !incomplete.contains(&function.symbol) {
            continue;
        }
        let span = function.span;
        let keep = function.params.iter().map(|param| param.local.0 as usize + 1).max().unwrap_or(0);
        function.locals.truncate(keep);
        let text = hir::Expr { ty: common.str_, kind: hir::ExprKind::Str(INCOMPLETE.to_string()), span };
        let stop = hir::Expr { ty: common.never, kind: hir::ExprKind::Builtin { which: hir::Builtin::Panic, args: vec![text] }, span };
        function.body = hir::Block { stmts: vec![hir::Stmt::Expr(stop)], span };
    }
}

/// The evaluator's program, compiled and linked.
fn link(build: &Build<'_>, toolchain: &Toolchain, emitted: &ember_codegen_c::Output) -> Result<PathBuf, String> {
    std::fs::create_dir_all(&build.c_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&build.obj_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&build.bin_dir).map_err(|e| e.to_string())?;
    let c_path = build.c_dir.join("comptime.c");
    std::fs::write(&c_path, &emitted.c_source).map_err(|e| e.to_string())?;
    let includes = vec![build.runtime.join("include")];
    let runtime_source = build.runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let runtime_input = ember_build::runtime_object_for_executable(
        toolchain,
        &runtime_source,
        &includes,
        build.profile,
        &ember_build::cache_root(),
    )
    .map_err(|e| e.to_string())?
    .unwrap_or(runtime_source);
    let mut sources = vec![c_path, runtime_input];
    let object_ext = if matches!(toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
    for (mode, source) in &emitted.relaxed_units {
        let unit = build.c_dir.join(format!("comptime.{}.c", mode.name()));
        std::fs::write(&unit, source).map_err(|e| e.to_string())?;
        let object = build.obj_dir.join(format!("comptime.{}.{object_ext}", mode.name()));
        ember_build::compile_relaxed_object_for_executable(
            toolchain,
            &unit,
            &includes,
            &object,
            build.profile,
            *mode == ember_types::FpMode::Fast,
        )
        .map_err(|e| e.to_string())?;
        sources.push(object);
    }
    let exe = build.bin_dir.join(if cfg!(windows) { "comptime.exe" } else { "comptime" });
    ember_build::compile_and_link_executable_with_roots(
        toolchain,
        &LinkRequest {
            sources: &sources,
            include_dirs: &includes,
            output: exe.clone(),
            profile: build.profile,
            obj_dir: build.obj_dir.clone(),
        },
        &[],
    )
    .map_err(|e| format!("cannot build the compile-time evaluator: {e}"))?;
    Ok(exe)
}

/// FNV-1a over `bytes`, from `seed`: a cache key, not a security hash.
fn fnv(bytes: &[u8], seed: u64) -> u64 {
    bytes.iter().fold(seed, |hash, &byte| (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3))
}

/// `[CT-2]`, `[CT-4]` — an input's present state: a file's bytes, or a
/// variable's value (`-` when it is not set).
fn input_hash(kind: &str, name: &str) -> u64 {
    match kind {
        "file" => std::fs::read(name).map_or(0, |bytes| fnv(&bytes, 0xcbf29ce484222325)),
        _ => match std::env::var_os(name) {
            Some(value) => fnv(value.to_string_lossy().as_bytes(), 0xcbf29ce484222325),
            None => fnv(b"-", 1),
        },
    }
}

/// `[CT-4]` — the value kept under `key`, if every input it read is as it
/// was then.
fn cached(dir: &Path, key: u64) -> Option<String> {
    let text = std::fs::read_to_string(dir.join(format!("{key:016x}"))).ok()?;
    let mut lines = text.lines();
    let value = lines.next()?.strip_prefix("value ")?.to_string();
    for line in lines {
        let mut parts = line.splitn(3, ' ');
        let (kind, hash, name) = (parts.next()?, parts.next()?, parts.next()?);
        if format!("{:016x}", input_hash(kind, name)) != hash {
            return None;
        }
    }
    Some(value)
}

/// `[CT-4]` — keep a value with the inputs the evaluator said it read.
fn keep(dir: &Path, key: u64, value: &str, stderr: &str) {
    let mut text = format!("value {value}\n");
    for line in stderr.lines() {
        if let Some(input) = line.strip_prefix("ember-comptime-input ")
            && let Some((kind, name)) = input.split_once(' ')
        {
            text.push_str(&format!("{kind} {:016x} {name}\n", input_hash(kind, name)));
        }
    }
    if std::fs::create_dir_all(dir).is_ok() {
        let _ = std::fs::write(dir.join(format!("{key:016x}")), text);
    }
}

/// The call chain the evaluator printed: the request's function first.
fn chain(stderr: &str) -> Vec<String> {
    stderr.lines().filter_map(|line| line.strip_prefix("ember-comptime-frame ")).map(str::to_string).collect()
}

/// `[CT-7]` — `E6004`: the panic's message and where it is, and the chain.
fn panicked(request: &ComptimeRequest, stderr: &str) -> Diagnostic {
    let line = stderr.lines().find(|line| line.starts_with("panic")).unwrap_or("panic");
    let (place, message) = match line.strip_prefix("panic at ").and_then(|rest| rest.split_once(": ")) {
        Some((place, message)) => (Some(place), message),
        None => (None, line.strip_prefix("panic: ").unwrap_or(line)),
    };
    let mut diagnostic = Diagnostic::error(codes::E6004, request.span, format!("evaluating {} panics: {message}", request.what))
        .primary_label(message.to_string());
    let frames = chain(stderr);
    if !frames.is_empty() {
        diagnostic = diagnostic.note(format!("the compile-time call chain: {}", frames.join(" -> ")));
    }
    if let Some(place) = place {
        diagnostic = diagnostic.note(format!("the panic is at {place}"));
    }
    diagnostic.note("a panic while compiling is an error where the evaluation is written [CT-7]")
}

/// `[CT-3]` — `E6001`: the limit passed, and where it was.
fn exhausted(request: &ComptimeRequest, stderr: &str, limits: Limits) -> Diagnostic {
    let (what, key, limit) = if stderr.contains("ember-comptime-limit heap") {
        ("bytes of heap", "max_heap_mb", limits.heap)
    } else {
        ("steps", "max_steps", limits.steps)
    };
    let mut diagnostic = Diagnostic::error(
        codes::E6001,
        request.span,
        format!("evaluating {} takes more than {limit} {what}", request.what),
    );
    let frames = chain(stderr);
    if !frames.is_empty() {
        diagnostic = diagnostic.note(format!("it was in: {}", frames.join(" -> ")));
    }
    diagnostic
        .help(format!("raise the limit with `{key} = …` under `[comptime]` in the manifest"))
        .note("each compile-time evaluation is limited in steps and heap [CT-3]")
}
