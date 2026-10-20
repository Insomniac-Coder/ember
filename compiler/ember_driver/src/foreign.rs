//! The foreign boundary as the driver reports it: each declaration of an `unsafe extern` block
//! with the facts it asserts about foreign code (`[FFI-10]`, `[TCB-1]`–`[TCB-3]`), for `ember tcb`
//! and `ember calls --foreign` (`[CLI-13]`).
//!
//! A declaration's facts are what a programmer wrote: its signature, `safe fn`, and each clause of
//! its `@ffi(…)` contract. Each is graded **asserted** (`[TCB-1]`): nothing written raises a grade
//! (`[TCB-2]`); a grade above asserted needs evidence, which a hand declaration has no way to claim.

use ember_ast::{AttrArg, Item, ItemKind};
use ember_span::{SourceMap, Span};
use ember_typeck::LoadedModule;

/// One fact a foreign declaration asserts, with where it is written and its grade.
pub struct ForeignFact {
    pub text: String,
    pub source: Span,
    pub grade: &'static str,
}

/// One declaration of an `unsafe extern` block.
pub struct ForeignDecl {
    pub module: String,
    /// The name the program uses, qualified by its module as the checker qualifies it.
    pub qualified: String,
    pub name: String,
    /// `fn`, `static` or `type`.
    pub kind: &'static str,
    pub abi: String,
    pub safe: bool,
    pub source: Span,
    pub facts: Vec<ForeignFact>,
}

/// Every foreign declaration of the program's own modules (the standard library's are its own
/// trusted base), in source order. `module_name` names a loaded module as the report shows it.
pub fn declarations(modules: &[LoadedModule], map: &SourceMap, module_name: &dyn Fn(&LoadedModule) -> String) -> Vec<ForeignDecl> {
    let mut out = Vec::new();
    for loaded in modules {
        if loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE) {
            continue;
        }
        let module = module_name(loaded);
        let prefix = if loaded.path.is_empty() { String::new() } else { format!("{}.", loaded.path.join(".")) };
        for item in &loaded.module.items {
            let ItemKind::ExternBlock(block) = &item.kind else { continue };
            for inner in &block.items {
                if let Some(decl) = declaration(inner, &block.abi, &module, &prefix, map) {
                    out.push(decl);
                }
            }
        }
    }
    out
}

fn declaration(item: &Item, abi: &str, module: &str, prefix: &str, map: &SourceMap) -> Option<ForeignDecl> {
    let text = |span: Span| -> String {
        let file = map.file(span.file);
        file.text.get(span.start as usize..span.end as usize).unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ")
    };
    let asserted = |text: String, source: Span| ForeignFact { text, source, grade: "asserted" };
    let (name, kind, safe, signature) = match &item.kind {
        ItemKind::Fn(decl) => (decl.name.name.to_string(), "fn", decl.is_safe, "its signature matches the C declaration"),
        ItemKind::Static(decl) => (decl.name.name.to_string(), "static", false, "its type matches the C object's"),
        ItemKind::TypeAlias(alias) => (alias.name.name.to_string(), "type", false, "it names a C type, used only behind pointers"),
        _ => return None,
    };
    // The symbol it binds, when `@ffi(link_name=…)` renames it (`[FFI-49]`).
    let symbol = item.attrs.iter()
        .filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("ffi"))
        .flat_map(|attr| &attr.args)
        .find_map(|arg| match arg {
            AttrArg::Named { name, value } if name.name.is("link_name") => Some(text(value.span)),
            _ => None,
        })
        .map(|symbol| format!(", C symbol {symbol}"))
        .unwrap_or_default();
    let mut facts = vec![asserted(format!("{signature} (`unsafe extern \"{abi}\"`{symbol})"), item.span)];
    if safe {
        facts.push(asserted("safe to call: a call within its contract cannot break memory safety (`safe fn`)".to_string(), item.span));
    }
    for attr in item.attrs.iter().filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("ffi")) {
        for arg in &attr.args {
            let span = match arg {
                AttrArg::Expr(expr) => expr.span,
                AttrArg::Named { name, value } => name.span.to(value.span),
            };
            let written = text(span);
            let meaning = match arg {
                AttrArg::Named { name, .. } if name.name.is("link_name") => continue,
                AttrArg::Named { name, .. } if name.name.is("effects") => "its effects",
                AttrArg::Expr(expr) => match &expr.kind {
                    ember_ast::ExprKind::Path { segments } if segments.len() == 1 && segments[0].name.is("immutable") => {
                        "a `const` C object that never changes"
                    }
                    ember_ast::ExprKind::Path { segments } if segments.len() == 1 && segments[0].name.is("deterministic") => {
                        "the same result on every run"
                    }
                    _ => "its contract",
                },
                AttrArg::Named { .. } => "its contract",
            };
            facts.push(asserted(format!("{meaning}: `{written}`"), span));
        }
    }
    Some(ForeignDecl {
        module: module.to_string(),
        qualified: format!("{prefix}{name}"),
        name,
        kind,
        abi: abi.to_string(),
        safe,
        source: item.span,
        facts,
    })
}

/// The foreign facts as JSON, for `ember tcb --json` and `ember calls --foreign --json`.
pub fn json(decl: &ForeignDecl, map: &SourceMap) -> serde_json::Value {
    serde_json::json!({
        "module": decl.module,
        "name": decl.name,
        "kind": decl.kind,
        "abi": decl.abi,
        "safe": decl.safe,
        "source": map.location(decl.source),
        "facts": decl.facts.iter().map(|fact| serde_json::json!({
            "fact": fact.text,
            "source": map.location(fact.source),
            "grade": fact.grade,
        })).collect::<Vec<_>>(),
    })
}

// -- `ember test --instrument-ffi` ([FFI-37]–[FFI-37d], [TCB-5], [TCB-6]) ------------------------

/// `[TCB-1]` — where evidence records live when the manifest's `[ffi] evidence` names nowhere.
pub fn default_evidence_dir() -> String {
    format!(".{}/ffi-evidence", ember_branding::SYMBOL_PREFIX)
}

/// The variable an instrumented test binary reads for the file its counts go to.
pub fn evidence_variable() -> String {
    format!("{}_FFI_EVIDENCE", ember_branding::SYMBOL_PREFIX.to_uppercase())
}

/// `[FFI-37d]` — the effect facts instrumentation can grade, by the bit the instrumentation
/// records each with, and the effect they are.
const EFFECT_FACTS: [(u32, &str, ember_hir::Effects); 4] = [
    (1, "Alloc", ember_hir::Effects::ALLOC),
    (2, "Lock", ember_hir::Effects::LOCK),
    (4, "Block", ember_hir::Effects::BLOCK),
    (8, "Io", ember_hir::Effects::IO),
];

/// One foreign function of an instrumented test binary: its name, the effect kinds its
/// `@ffi(effects=[…])` states it has (`None` when it states none: then it has every effect, and
/// no absence to test), and its declaration.
struct InstrumentedFunction {
    name: String,
    declared: Option<u32>,
    span: Span,
}

/// `[FFI-37]` — what an instrumented test binary counts, with what the program declares of it.
pub struct InstrumentedRun {
    /// `[TCB-5]` — per source file `import c` wrote: the header and overlay its declarations rest on.
    pub imported: std::collections::HashMap<ember_span::FileId, Vec<(String, std::path::PathBuf)>>,
    functions: Vec<InstrumentedFunction>,
    sites: Vec<(u32, String)>,
    dir: std::path::PathBuf,
    package: String,
    exe: std::path::PathBuf,
    toolchain: String,
}

impl InstrumentedRun {
    /// The functions in the order the code generator numbered them: the program's bodies that
    /// are foreign declarations.
    pub fn new(
        instrument: &ember_codegen_c::FfiInstrumentOutput,
        bodies: &[ember_mir::Body],
        dir: std::path::PathBuf,
        package: &str,
        exe: &std::path::Path,
        toolchain: String,
    ) -> InstrumentedRun {
        let functions = bodies.iter().filter_map(|body| {
            let effects = body.contracts.foreign?;
            let stated = !effects.contains(ember_hir::Effects(ember_hir::Effects::ALL.0 & !ember_hir::Effects::NONDET.0));
            let declared = stated.then(|| EFFECT_FACTS.iter().filter(|(_, _, effect)| effects.contains(*effect)).map(|(bit, _, _)| bit).sum());
            Some(InstrumentedFunction { name: body.name.clone(), declared, span: body.span })
        }).collect::<Vec<_>>();
        debug_assert_eq!(functions.len(), instrument.functions.len());
        InstrumentedRun { imported: Default::default(), functions, sites: instrument.sites.clone(), dir, package: package.to_string(), exe: exe.to_path_buf(), toolchain }
    }
}

/// What the test runs observed, merged.
#[derive(Default)]
struct Observed {
    intercepted: String,
    calls: Vec<u64>,
    reached: Vec<u32>,
    module: Vec<String>,
    site_hits: Vec<u64>,
}

fn read_counts(files: &[std::path::PathBuf], functions: usize, sites: usize) -> Observed {
    let mut observed = Observed { calls: vec![0; functions], reached: vec![0; functions], module: vec![String::new(); functions], site_hits: vec![0; sites], ..Observed::default() };
    for file in files {
        let Ok(text) = std::fs::read_to_string(file) else { continue };
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("intercepted ") {
                observed.intercepted = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("function ") {
                let mut parts = rest.splitn(4, ' ');
                let (Some(index), Some(calls), Some(reached)) = (parts.next(), parts.next(), parts.next()) else { continue };
                let (Ok(index), Ok(calls), Ok(reached)) = (index.parse::<usize>(), calls.parse::<u64>(), reached.parse::<u32>()) else { continue };
                if index < functions {
                    observed.calls[index] += calls;
                    observed.reached[index] |= reached;
                    if let Some(module) = parts.next().filter(|module| !module.is_empty()) {
                        observed.module[index] = module.to_string();
                    }
                }
            } else if let Some(rest) = line.strip_prefix("site ") {
                let mut parts = rest.split(' ');
                if let (Some(Ok(index)), Some(Ok(hits))) = (parts.next().map(str::parse::<usize>), parts.next().map(str::parse::<u64>))
                    && index < sites
                {
                    observed.site_hits[index] += hits;
                }
            }
        }
    }
    observed
}

/// `[FFI-37]`–`[FFI-37d]`, `[TCB-5]` — grade each declared effect fact by what the runs observed:
/// a fact a run contradicted is `E5053`; one whose function no test called is `W5054` and counts
/// for nothing; the rest are `instrumented` — no counterexample on the paths exercised, partial
/// when not every call site ran, never "holds". Prints the grades, writes the evidence record
/// (its identities, its environment) and reports the diagnostics; true when one is `E5053`.
pub fn instrument_report(
    run: &InstrumentedRun,
    files: &[std::path::PathBuf],
    map: &SourceMap,
    options: &crate::Options,
) -> Result<bool, String> {
    let observed = read_counts(files, run.functions.len(), run.sites.len());
    let mut sink = ember_diag::Sink::new();
    let mut lines = Vec::new();
    let mut records = Vec::new();
    let mut contradicted = false;
    for (index, function) in run.functions.iter().enumerate() {
        let Some(declared) = function.declared else { continue };
        let facts: Vec<&(u32, &str, ember_hir::Effects)> = EFFECT_FACTS.iter().filter(|(bit, _, _)| declared & bit == 0).collect();
        if facts.is_empty() {
            continue;
        }
        let total_sites = run.sites.iter().filter(|(owner, _)| *owner as usize == index).count();
        let site_records: Vec<serde_json::Value> = run.sites.iter().enumerate()
            .filter(|(_, (owner, _))| *owner as usize == index)
            .map(|(site, (_, at))| serde_json::json!({ "at": at, "hits": observed.site_hits[site] }))
            .collect();
        let ran_sites = run.sites.iter().enumerate().filter(|(site, (owner, _))| *owner as usize == index && observed.site_hits[*site] > 0).count();
        let calls = observed.calls[index];
        let fact_names: Vec<String> = facts.iter().map(|(_, name, _)| format!("no {name}")).collect();
        let mut grades = Vec::new();
        let summary;
        if calls == 0 {
            sink.emit(
                ember_diag::Diagnostic::warning(ember_diag::codes::W5054, function.span,
                    format!("`{}`'s effect facts were never exercised: no test called it", function.name))
                    .note(format!("{}: unexercised, so they count for nothing [FFI-37b]", fact_names.join(", "))),
            );
            for (_, name, _) in &facts {
                grades.push(serde_json::json!({ "effect": name, "grade": "unexercised" }));
            }
            summary = format!("{}: unexercised (W5054)", fact_names.join(", "));
        } else {
            let mut parts = Vec::new();
            for (bit, name, _) in &facts {
                if observed.reached[index] & bit != 0 {
                    contradicted = true;
                    sink.emit(
                        ember_diag::Diagnostic::error(ember_diag::codes::E5053, function.span,
                            format!("an instrumented run contradicted `{}`'s effects: it reached {name}", function.name))
                            .help(format!("add `{name}` to its `@ffi(effects=[…])`, if the function does that"))
                            .note("a run of `test --instrument-ffi` saw the call reach an intercepted entry point of that kind [FFI-37c]"),
                    );
                    grades.push(serde_json::json!({ "effect": name, "grade": "contradicted" }));
                    parts.push(format!("no {name}: contradicted (E5053)"));
                } else {
                    let partial = ran_sites < total_sites;
                    grades.push(serde_json::json!({ "effect": name, "grade": "instrumented", "partial": partial }));
                    parts.push(format!("no {name}: instrumented{}", if partial { ", partial" } else { "" }));
                }
            }
            summary = parts.join("; ");
        }
        lines.push(format!("  `{}`: {calls} call{} from {ran_sites} of {total_sites} call site{}; {summary}",
            function.name, if calls == 1 { "" } else { "s" }, if total_sites == 1 { "" } else { "s" }));
        let file = map.file(function.span.file);
        let library = &observed.module[index];
        let imported_from: Vec<serde_json::Value> = run
            .imported
            .get(&function.span.file)
            .into_iter()
            .flatten()
            .map(|(role, path)| serde_json::json!({
                "role": role,
                "path": path.display().to_string(),
                "hash": std::fs::read(path).ok().map(|bytes| ember_build::content_hash(&bytes)),
            }))
            .collect();
        records.push(serde_json::json!({
            "name": function.name,
            "declared_in": file.path.display().to_string(),
            "source_hash": ember_build::content_hash(file.text.as_bytes()),
            "imported_from": imported_from,
            "library": {
                "path": library,
                "hash": std::fs::read(library).ok().map(|bytes| ember_build::content_hash(&bytes)),
            },
            "calls": calls,
            "sites": site_records,
            "facts": grades,
        }));
    }
    let unobservable = "an allocation, lock, wait or I/O through anything outside the intercepted set (a custom pool, a driver, a module loaded after the start, a system call made directly) is structurally unobservable, and counts for nothing";
    let machine = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_default();
    let recorded = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|time| time.as_secs()).unwrap_or(0);
    let record = serde_json::json!({
        "schema": 1,
        "package": run.package,
        "intercepted": observed.intercepted,
        "unobservable": unobservable,
        "identity": {
            "toolchain": run.toolchain,
            "test_binary": {
                "path": run.exe.display().to_string(),
                "hash": std::fs::read(&run.exe).ok().map(|bytes| ember_build::content_hash(&bytes)),
            },
        },
        "environment": { "machine": machine, "recorded": recorded },
        "functions": records,
    });
    std::fs::create_dir_all(&run.dir).map_err(|error| format!("cannot write the evidence record: {error}"))?;
    let path = run.dir.join(format!("{}.json", run.package));
    std::fs::write(&path, serde_json::to_string_pretty(&record).map_err(|error| error.to_string())?)
        .map_err(|error| format!("cannot write the evidence record `{}`: {error}", path.display()))?;
    if !options.json {
        println!("\nforeign effects, instrumented [FFI-37]:");
        println!("  intercepted: {}", observed.intercepted);
        println!("  {unobservable} [FFI-37a]");
        println!("  ownership, nullability, lifetime, aliasing and exception facts stay asserted [FFI-37d]");
        for line in &lines {
            println!("{line}");
        }
        println!("  evidence: {}", path.display());
    }
    crate::report(&sink, map, options);
    Ok(contradicted)
}

/// `[TCB-5]`, `[TCB-6]` — a package's evidence record, read back for `ember tcb`: whether its
/// identity inputs still hold (the test binary and the C toolchain here; each function's
/// declaring source and library below), and where it was measured (an environment input, which
/// is reported and never makes it stale).
pub struct Evidence {
    pub path: std::path::PathBuf,
    record: serde_json::Value,
    stale: Vec<String>,
    elsewhere: Option<String>,
}

/// The evidence record of the package whose entry is `input`, if a run of
/// `test --instrument-ffi` wrote one.
pub fn load_evidence(input: &std::path::Path) -> Option<Evidence> {
    let start = std::fs::canonicalize(input).ok()?;
    let manifest = start.ancestors().map(|dir| dir.join(ember_branding::MANIFEST)).find(|path| path.is_file())?;
    let root = manifest.parent()?;
    let text = std::fs::read_to_string(&manifest).ok()?;
    let dir = crate::manifest_string(&text, "ffi", "evidence").unwrap_or_else(default_evidence_dir);
    let stem = input.file_stem()?.to_string_lossy().into_owned();
    let path = root.join(dir).join(format!("{stem}.json"));
    let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
    let mut stale = Vec::new();
    let file_hash = |path: &str| std::fs::read(path).ok().map(|bytes| ember_build::content_hash(&bytes));
    let binary = &record["identity"]["test_binary"];
    if binary["path"].as_str().and_then(file_hash).as_deref() != binary["hash"].as_str() {
        stale.push("the test binary".to_string());
    }
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = ember_build::Toolchain::detect(requested.as_deref()).ok().map(|toolchain| toolchain.identity());
    if toolchain.as_deref() != record["identity"]["toolchain"].as_str() {
        stale.push("the C toolchain".to_string());
    }
    let machine = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_default();
    let recorded_on = record["environment"]["machine"].as_str().unwrap_or("").to_string();
    let elsewhere = (recorded_on != machine).then_some(recorded_on);
    Some(Evidence { path, record, stale, elsewhere })
}

/// `[TCB-1]` — whether `evidence` backs an `instrumented` grade for the foreign function `name`,
/// declared in a file whose text hashes to `source_hash`: the record is for this test binary and
/// toolchain, names the function from that same declaration, every file it rests on (its library,
/// header and overlay) is unchanged, and every effect fact it graded is `instrumented`.
pub fn backs_instrumented(evidence: &Evidence, name: &str, source_hash: &str) -> bool {
    let file_hash = |path: &str| std::fs::read(path).ok().map(|bytes| ember_build::content_hash(&bytes));
    let Some(function) = evidence.record["functions"].as_array().into_iter().flatten().find(|function| function["name"] == name) else {
        return false;
    };
    let unchanged = |identity: &serde_json::Value| {
        let path = identity["path"].as_str().unwrap_or("");
        path.is_empty() || file_hash(path).as_deref() == identity["hash"].as_str()
    };
    let facts = function["facts"].as_array().map(|facts| facts.as_slice()).unwrap_or(&[]);
    evidence.stale.is_empty()
        && function["source_hash"].as_str() == Some(source_hash)
        && unchanged(&function["library"])
        && function["imported_from"].as_array().into_iter().flatten().all(unchanged)
        && !facts.is_empty()
        && facts.iter().all(|fact| fact["grade"] == "instrumented")
}

/// `[FFI-37c]`, `[TCB-5]`, `[TCB-6]` — what the evidence says of the effect facts of `decl`: each
/// graded fact (instrumented, partial when not every call site ran; contradicted; unexercised),
/// or why the record no longer counts. `None` when the record holds nothing of `decl`.
pub fn evidence_lines(decl: &ForeignDecl, evidence: &Evidence, map: &SourceMap) -> Option<Vec<String>> {
    let function = evidence.record["functions"].as_array()?.iter().find(|function| function["name"] == decl.qualified)?;
    let file_hash = |path: &str| std::fs::read(path).ok().map(|bytes| ember_build::content_hash(&bytes));
    let mut stale = evidence.stale.clone();
    let source = map.file(decl.source.file);
    if function["source_hash"].as_str() != Some(ember_build::content_hash(source.text.as_bytes()).as_str()) {
        stale.push(format!("its declaration in {}", source.path.display()));
    }
    let library = function["library"]["path"].as_str().unwrap_or("");
    if !library.is_empty() && file_hash(library).as_deref() != function["library"]["hash"].as_str() {
        stale.push(format!("its library {library}"));
    }
    // `[TCB-5]` — an imported declaration's header and overlay.
    for identity in function["imported_from"].as_array().into_iter().flatten() {
        let path = identity["path"].as_str().unwrap_or("");
        if file_hash(path).as_deref() != identity["hash"].as_str() {
            stale.push(format!("its {} {path}", identity["role"].as_str().unwrap_or("source")));
        }
    }
    let mut lines = Vec::new();
    if !stale.is_empty() {
        lines.push(format!("evidence stale, so its effect facts stay asserted: {} changed since {} [TCB-6]",
            stale.join(", "), evidence.path.display()));
        return Some(lines);
    }
    let sites = function["sites"].as_array().map(|sites| sites.as_slice()).unwrap_or(&[]);
    let ran = sites.iter().filter(|site| site["hits"].as_u64().unwrap_or(0) > 0).count();
    let grades: Vec<String> = function["facts"].as_array().map(|facts| facts.as_slice()).unwrap_or(&[]).iter().map(|fact| {
        let effect = fact["effect"].as_str().unwrap_or("?");
        match (fact["grade"].as_str(), fact["partial"].as_bool()) {
            (Some("instrumented"), Some(true)) => format!("no {effect}: instrumented, partial ({ran} of {} call sites ran)", sites.len()),
            (Some("instrumented"), _) => format!("no {effect}: instrumented"),
            (Some(grade), _) => format!("no {effect}: {grade}"),
            (None, _) => format!("no {effect}: asserted"),
        }
    }).collect();
    lines.push(format!("by evidence {}: {} [FFI-37c]", evidence.path.display(), grades.join(", ")));
    if grades.iter().any(|grade| grade.contains("instrumented")) {
        lines.push("instrumented: no counterexample was observed on the paths exercised, which is not that the fact holds".to_string());
    }
    if let Some(machine) = &evidence.elsewhere {
        lines.push(format!("measured on {}, not this machine: an environment input, reported, not a reason the record is stale [TCB-6]",
            if machine.is_empty() { "an unnamed machine" } else { machine }));
    }
    Some(lines)
}

/// The facts of one declaration, as `ember tcb` and `ember calls --foreign` print them, with what
/// an evidence record says of its effect facts.
pub fn print(decl: &ForeignDecl, map: &SourceMap, indent: &str, evidence: Option<&Evidence>) {
    let safety = if decl.kind == "fn" { if decl.safe { " safe" } else { " unsafe to call" } } else { "" };
    println!("{indent}{}  {} `{}`{safety}", map.location(decl.source), decl.kind, decl.name);
    for fact in &decl.facts {
        println!("{indent}  {}  [{}]  {}", fact.text, fact.grade, map.location(fact.source));
    }
    if decl.kind == "fn" && !decl.facts.iter().any(|fact| fact.text.starts_with("its effects")) {
        println!("{indent}  its effects: none stated, so every effect [EFF-3]");
    }
    for line in evidence.and_then(|evidence| evidence_lines(decl, evidence, map)).unwrap_or_default() {
        println!("{indent}  {line}");
    }
}
