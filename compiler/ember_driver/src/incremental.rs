//! `[BLD-1]`, `[BLD-2]`, `[BLD-8]` (ODR-257) — the module as the unit of front-end caching, and the
//! function as the unit of checking within it.
//!
//! After a check that reported no error, each module's *checked form* is kept in
//! `target/<profile>/incremental/`, in parts: each function with a body of its own (its MIR as the
//! checks leave it — drops elaborated, each callable's region summary installed — the full region
//! contract of each of its bodies and of its closures', what its body asked of the program, the
//! generics' instances it calls (`ember_typeck::Kept`), and the warnings checking it reported);
//! each generic's instance its bodies reach, likewise; and the rest of the module's check. Each
//! part records the facts of the bodies it calls as they were (`[BLD-2]`: a body's region contract
//! — a view result's sources, a method's field set, what a callee stores — and whether its result
//! can be followed).
//!
//! The next build keeps a module's parts while what they were checked against has not changed:
//!
//! * the compiler, the command, the profile, the options that change a check, the package's
//!   manifest, and the declarations every module sees without importing anything — each `extend`
//!   and each class (the *key*);
//! * the module's declarations — its source with the bodies of its ordinary (non-generic)
//!   functions left out — and those of every module it imports, directly or not: what it can name,
//!   call and instantiate;
//! * the text each part's spans point at, wherever it is now (`ember_span::codec::Relocation`): a
//!   part whose own body was edited is checked again, and a body edited elsewhere moves the rest;
//! * the facts of the bodies each part calls: a build assumes them, and once the bodies checked
//!   again are, each kept part whose facts changed is checked again too (`Attempts::again`).
//!
//! A module whose source is unchanged and whose parts all hold is kept whole: the checker makes its
//! functions' signatures only, finds again the types its kept form names, and asks again for the
//! instances its bodies asked for, which it checks as it checks every instance. One whose
//! declarations are unchanged is checked in part (`[BLD-8]`): the bodies whose parts hold are not
//! checked; the rest of it is. The kept MIR joins the program's after lowering; every pass over the
//! whole program (the effects, the `@static_safe` verdict, the specialisations, …) reads it as it
//! reads any body, while a pass that only checks one body leaves it alone (`Body::kept`). What
//! checking it reported is reported again, where a clean build reports it. A check that asks the
//! compile-time evaluator for a value is made whole, since the evaluator runs function bodies.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use ember_analysis::regions::StoreSummary;
use ember_analysis::{BorrowFacts, CallRegionContract};
use ember_diag::{Diagnostic, Severity, Sink};
use ember_mir::Body;
use ember_span::codec::Relocation;
use ember_span::{FileId, SourceMap, Span};
use ember_types::TypeTable;
use ember_types::codec::{Found, TypeRecord};
use ember_typeck::{CheckOutput, Kept, LoadedModule};
use serde::{Deserialize, Serialize};

/// A record of another schema is one this compiler cannot read: it is not used. Schema 4
/// (ODR-281): a record names its imports at their reach hashes, where schema 3 named every module
/// it reached at its declarations.
const SCHEMA: u32 = 4;
const EXTENSION: &str = "emck";

type Hash = [u8; 32];

fn hash(bytes: &[u8]) -> Hash {
    *blake3::hash(bytes).as_bytes()
}

/// What one attempt at a compilation learned for the next (`compile` makes them until one is
/// not told to try `again`).
#[derive(Default)]
pub(crate) struct Attempts {
    pub again: bool,
    /// Modules to check whole whatever their records say: their records did not hold.
    pub forced: HashSet<String>,
    /// Parts to check again whatever their records say, by module and part (`Part::key`): the
    /// facts they used changed. A module with one of these is not kept whole.
    pub forced_parts: HashSet<(String, String)>,
    /// Generics' instances to make and check again, by symbol.
    pub forced_instances: HashSet<String>,
    /// Check every module: the compile-time evaluator needs the bodies.
    pub whole: bool,
}

/// What a module's check left for the next build.
#[derive(Serialize, Deserialize)]
struct Record {
    schema: u32,
    name: String,
    key: Hash,
    source: Hash,
    declarations: Hash,
    /// Each module it imports: its name and its reach hash (`Plan::reach`) when this was kept.
    deps: Vec<(String, Hash)>,
    /// The files its spans point into (numbered as its spans name them).
    files: Vec<FileRef>,
    /// The types its parts name (`ember_types::codec::TypeRecord`), written with its files.
    types: Vec<u8>,
    /// What the module's check made outside its functions' parts: used when it is kept whole.
    module: Part,
    /// Its functions with bodies of their own, each with its body (its place among its file's
    /// pieces); then the generics' instances its bodies reach (no body of the module's).
    units: Vec<(Option<u32>, Part)>,
    /// Its module interface artifact (`ember_build::interface`, `[BLD-2]`): the module's name there,
    /// what it imports directly or not, and its interface hash.
    interface: (String, Vec<String>, [u8; 32]),
}

/// A file a record's spans point into: its path, and what of it the record holds to.
#[derive(Serialize, Deserialize)]
struct FileRef {
    path: String,
    kind: FileKind,
}

#[derive(Serialize, Deserialize)]
enum FileKind {
    /// A module's file: its declarations (`Shape::declarations`) when the record was kept, and the
    /// pieces of it the record's spans lie in.
    Module { declarations: Hash, pieces: Vec<Piece> },
    /// Another file (one a C import read): its text's hash.
    Whole(Hash),
}

/// A piece of a module's file (`Shape::pieces`): its place among them, where it began and how
/// long it was, and, for a body, its text's hash (the text between bodies is the declarations').
#[derive(Serialize, Deserialize, Clone, Copy)]
struct Piece {
    index: u32,
    start: u32,
    length: u32,
    body: Option<Hash>,
}

/// One part of a record, read on its own: the pieces its spans lie in (by file and place in that
/// file's `pieces`), its bodies with their facts, the facts of the bodies it calls as they were,
/// and its checked form (`Form`), written with the record's files and types.
#[derive(Serialize, Deserialize, Default)]
struct Part {
    pieces: Vec<(u32, u32)>,
    facts: Vec<(String, Hash)>,
    uses: Vec<(String, Hash)>,
    form: Vec<u8>,
}

impl Part {
    /// The part's name among its module's: its first body's symbol.
    fn key(&self) -> &str {
        self.facts.first().map_or("", |(symbol, _)| symbol.as_str())
    }
}

/// A part's checked form.
#[derive(Serialize, Deserialize, Default)]
struct Form {
    bodies: Vec<Body>,
    /// Each body's region contract, what it stores where its callers' regions do not reach, and
    /// whether no caller's regions can follow its result (`ember_analysis::BorrowFacts`).
    contracts: Vec<(String, CallRegionContract)>,
    stores: Vec<(String, StoreSummary)>,
    invalid: Vec<String>,
    /// What its bodies asked of the program.
    kept: Kept,
    /// The warnings the type check of its bodies reported, and those the checks of their MIR
    /// did: a build reports each where a clean build reaches the check that does.
    diagnostics: Vec<Diagnostic>,
    body_diagnostics: Vec<Diagnostic>,
}

/// What the kept bodies give the checks of the rest: each one's region contract, and what borrow
/// checking inferred of it.
#[derive(Default)]
pub(crate) struct KeptFacts {
    pub contracts: HashMap<String, CallRegionContract>,
    pub borrow: BorrowFacts,
    /// The kept modules' interface artifacts, as their records know them.
    pub interfaces: std::collections::BTreeMap<String, ember_build::interface::KnownInterface>,
}

/// A module whose record this build uses, whole or in part.
struct Clean {
    module: usize,
    record: Record,
    /// The record's files as this run finds them, each with how its offsets move.
    files: Vec<(FileId, Relocation)>,
    /// Whether the module is kept whole; else which of its units are kept, by place in `units`.
    whole: bool,
    units: Vec<bool>,
}

/// A kept part this build uses: its record's module, what it records, and its form once read.
struct Used {
    module: usize,
    facts: Vec<(String, Hash)>,
    uses: Vec<(String, Hash)>,
    kept: Kept,
    diagnostics: Vec<Diagnostic>,
    body_diagnostics: Vec<Diagnostic>,
}

/// One build's use of the kept checked forms.
pub(crate) struct Plan {
    dir: PathBuf,
    key: Hash,
    names: Vec<String>,
    shapes: Vec<Shape>,
    /// Each module's place by its file.
    module_of_file: HashMap<FileId, usize>,
    /// Each module's imports, by their places in the loaded list.
    imports: Vec<Vec<usize>>,
    /// ODR-281 — each module's reach hash: of its declarations and of those of every module it
    /// reaches by its imports (`ember_build::interface::component_hashes`). A record names its
    /// imports at theirs, so its size is its imports', not every module's below it.
    reach: Vec<Hash>,
    clean: Vec<Clean>,
    /// The modules kept whole, and those kept in part with the bodies (their spans now) kept.
    whole_set: HashSet<usize>,
    partial: HashMap<usize, HashSet<Span>>,
    /// The types of the clean modules' records, in `clean`'s order (what `ember_typeck::Reuse`
    /// finds again).
    pub types: Vec<TypeRecord>,
    /// The bodies of ordinary functions this build keeps, by file, sorted.
    kept_bodies: HashMap<FileId, Vec<(u32, u32)>>,
    /// Each clean module's parts this build uses, read by `kept`: its module part (kept whole), its
    /// kept own units, its instances; each with its form.
    read: std::sync::Mutex<Vec<Option<Vec<(Used, Form, Option<String>)>>>>,
    /// The kept parts this build uses, after `inject`: own parts by their first body's symbol,
    /// instances by theirs.
    used_parts: HashMap<String, Used>,
    used_instances: HashMap<String, Used>,
    /// The kept bodies' symbols that are not generics' instances (`inject`).
    kept_functions: HashSet<String>,
}

/// Whether a command, with these options, may use and keep checked forms: a plain check, build or
/// run (ODR-257: one checked form serves the three, the program's checks being the same in each).
/// A report, an output of a stage, the evaluator, `ember lint`, a test build (whose tests are marked
/// before the checks) and the like read more than a check reports, so they check every module.
pub(crate) fn applies(command: &str, plain: bool) -> bool {
    matches!(command, "check" | "build" | "run") && plain && std::env::var_os(ember_branding::no_incremental_var()).is_none()
}

impl Plan {
    /// The plan for this build: which modules' kept forms it uses, and which parts of them.
    /// `options` — the options that change what a check reports, spelled; `manifest` — the
    /// package's manifest, if it has one.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn open(
        profile_root: &Path,
        input: &Path,
        command: &str,
        options: &str,
        manifest: Option<&str>,
        modules: &[LoadedModule],
        map: &SourceMap,
        attempts: &Attempts,
    ) -> Plan {
        let package = std::path::absolute(input).unwrap_or_else(|_| input.to_path_buf());
        let dir = profile_root.join("incremental").join(hex(&hash(package.to_string_lossy().as_bytes()))[..32].to_string());
        let names: Vec<String> = modules.iter().map(|loaded| module_name(loaded, map)).collect();
        let sources: Vec<Hash> = modules.iter().map(|loaded| hash(map.file(loaded.module.span.file).text.as_bytes())).collect();
        let shapes: Vec<Shape> = modules.iter().map(|loaded| declarations(loaded, map)).collect();
        let open: Vec<String> = shapes.iter().map(|shape| shape.open.clone()).collect();
        // `ember check`, `ember build` and `ember run` check alike (`applies`): one key.
        let command = if matches!(command, "build" | "run") { "check" } else { command };
        let key = hash(
            format!(
                "{SCHEMA}\0{}\0{command}\0{options}\0{}\0{}",
                ember_build::interface::compiler_identity(),
                manifest.unwrap_or(""),
                hex(&open_world_of(&names, &open))
            )
            .as_bytes(),
        );
        let module_of_file = modules.iter().enumerate().map(|(index, loaded)| (loaded.module.span.file, index)).collect();
        let imports = direct_imports(modules);
        let reach = {
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let declarations: Vec<Hash> = shapes.iter().map(|shape| shape.declarations).collect();
            ember_build::interface::component_hashes(&names, &declarations, &imports, 0..modules.len())
                .expect("each import is a loaded module")
                .into_iter()
                .map(|hash| hash.expect("every module is a root"))
                .collect()
        };
        let mut plan = Plan {
            dir,
            key,
            names,
            shapes,
            module_of_file,
            imports,
            reach,
            clean: Vec::new(),
            whole_set: HashSet::new(),
            partial: HashMap::new(),
            types: Vec::new(),
            kept_bodies: HashMap::new(),
            read: Default::default(),
            used_parts: HashMap::new(),
            used_instances: HashMap::new(),
            kept_functions: HashSet::new(),
        };
        if attempts.whole {
            return plan;
        }
        let by_name: HashMap<&str, usize> = plan.names.iter().enumerate().map(|(index, name)| (name.as_str(), index)).collect();
        for index in 0..modules.len() {
            let Some(record) = plan.read_record(index) else { continue };
            if record.key != plan.key || record.declarations != plan.shapes[index].declarations || attempts.forced.contains(&plan.names[index]) {
                continue;
            }
            // Each import as the record knows it: the same modules, reaching modules declaring the same.
            let now: BTreeSet<&str> = plan.imports[index].iter().map(|dep| plan.names[*dep].as_str()).collect();
            let then: BTreeSet<&str> = record.deps.iter().map(|(name, _)| name.as_str()).collect();
            let deps_hold = now == then
                && record.deps.iter().all(|(name, reach)| by_name.get(name.as_str()).is_some_and(|&dep| plan.reach[dep] == *reach));
            if !deps_hold {
                continue;
            }
            let own = modules[index].module.span.file;
            let Some(held) = record.files.iter().map(|file| plan.find_file(file, map)).collect::<Option<Vec<_>>>() else { continue };
            // Which parts hold: each piece it lies in is where it is now, unchanged; and it is not
            // to be checked again.
            let name = &plan.names[index];
            let holds = |part: &Part| {
                part.pieces.iter().all(|&(file, piece)| held[file as usize].1[piece as usize].is_some())
                    && !attempts.forced_parts.contains(&(name.clone(), part.key().to_string()))
            };
            let units: Vec<bool> = record
                .units
                .iter()
                .map(|(body, part)| holds(part) && (body.is_some() || !attempts.forced_instances.contains(part.key())))
                .collect();
            let whole = record.source == sources[index]
                && holds(&record.module)
                && record.units.iter().zip(&units).all(|((body, _), holds)| *holds || body.is_none());
            let own_kept: Vec<u32> = record.units.iter().zip(&units).filter(|((_, _), holds)| **holds).filter_map(|((body, _), _)| *body).collect();
            if !whole && own_kept.is_empty() {
                continue;
            }
            // The bodies kept, where they are now; a body of this module checked again is no
            // piece a kept span moves into (a closure written there is made by its check).
            let own_number = held.iter().position(|(file, _)| *file == own);
            let kept_now: Vec<(u32, u32)> = match whole {
                true => plan.shapes[index].bodies.clone(),
                false => own_kept.iter().map(|&piece| plan.shapes[index].piece(piece)).collect(),
            };
            let files: Vec<(FileId, Relocation)> = held
                .iter()
                .enumerate()
                .map(|(number, (file, pieces))| {
                    let FileKind::Module { pieces: then, .. } = &record.files[number].kind else { return (*file, Relocation::identity()) };
                    let moved = then.iter().zip(pieces).filter_map(|(piece, now)| {
                        let now = (*now)?;
                        let dropped = Some(number) == own_number && piece.body.is_some() && !whole && !own_kept.contains(&piece.index);
                        (!dropped).then_some((piece.start, piece.length, now))
                    });
                    (*file, Relocation::pieces(moved.collect()))
                })
                .collect();
            let types = ember_span::codec::decode_relocated(files.clone(), true, || ember_span::codec::from_bytes::<TypeRecord>(&record.types));
            let Ok(types) = types else { continue };
            let mut sorted = kept_now.clone();
            sorted.sort_unstable();
            plan.kept_bodies.entry(own).or_default().extend(sorted);
            if whole {
                plan.whole_set.insert(index);
            } else {
                plan.partial.insert(index, kept_now.iter().map(|&(start, end)| Span::new(own, start, end)).collect());
            }
            plan.clean.push(Clean { module: index, record, files, whole, units });
            plan.types.push(types);
        }
        for bodies in plan.kept_bodies.values_mut() {
            bodies.sort_unstable();
        }
        plan.read = std::sync::Mutex::new((0..plan.clean.len()).map(|_| None).collect());
        plan
    }

    /// A record's file as this run finds it: its file, and for each of the record's pieces of it
    /// where that piece is now, when it is there unchanged.
    fn find_file(&self, file: &FileRef, map: &SourceMap) -> Option<(FileId, Vec<Option<u32>>)> {
        let id = map.find(Path::new(&file.path))?;
        let text = &map.file(id).text;
        match &file.kind {
            FileKind::Whole(kept) => (hash(text.as_bytes()) == *kept).then(|| (id, Vec::new())),
            FileKind::Module { declarations, pieces } => {
                let module = *self.module_of_file.get(&id)?;
                let shape = &self.shapes[module];
                if shape.declarations != *declarations {
                    return Some((id, vec![None; pieces.len()]));
                }
                let now = shape.pieces(text.len() as u32);
                let held = pieces
                    .iter()
                    .map(|piece| {
                        let &(start, end) = now.get(piece.index as usize)?;
                        let same = end - start == piece.length
                            && piece.body.is_none_or(|kept| hash(&text.as_bytes()[start as usize..end as usize]) == kept);
                        same.then_some(start)
                    })
                    .collect();
                Some((id, held))
            }
        }
    }

    /// The modules kept whole, by their places in the loaded list.
    pub(crate) fn whole(&self) -> &HashSet<usize> {
        &self.whole_set
    }

    /// `[BLD-8]` — the modules checked in part, each with the bodies (their spans now) kept.
    pub(crate) fn partial(&self) -> &HashMap<usize, HashSet<Span>> {
        &self.partial
    }

    /// Whether `span` is in the body of an ordinary function this build keeps (`ember_typeck::Reuse::kept_closure`).
    pub(crate) fn keeps_body_at(&self, span: Span) -> bool {
        let Some(bodies) = self.kept_bodies.get(&span.file) else { return false };
        let after = bodies.partition_point(|&(start, _)| start <= span.start);
        bodies[..after].last().is_some_and(|&(start, end)| start <= span.start && span.end <= end)
    }

    /// `[BLD-9]` — the modules this build checks, whole or in part, by name, for `--timings`.
    pub(crate) fn checked_names(&self) -> String {
        let mut names: Vec<&str> = (0..self.names.len())
            .filter(|index| !self.whole_set.contains(index))
            .map(|index| self.names[index].as_str())
            .collect();
        names.sort();
        names.join(", ")
    }

    /// `[BLD-8]`, `[BLD-9]` — the functions this build checks in the modules it checks in part, by
    /// symbol, for `--timings`.
    pub(crate) fn checked_functions(&self, checked: &CheckOutput) -> String {
        let mut symbols: Vec<&str> = self
            .partial
            .keys()
            .filter_map(|&module| checked.modules[module].as_ref())
            .flat_map(|check| &check.parts)
            .filter(|part| !part.kept)
            .filter_map(|part| part.functions.first().map(String::as_str))
            .collect();
        symbols.sort();
        symbols.join(", ")
    }

    fn path(&self, index: usize) -> PathBuf {
        self.dir.join(format!("{}.{EXTENSION}", &hex(&hash(self.names[index].as_bytes()))[..32]))
    }

    fn read_record(&self, index: usize) -> Option<Record> {
        let bytes = std::fs::read(self.path(index)).ok()?;
        let record: Record = ember_span::codec::from_bytes(&bytes).ok()?;
        (record.schema == SCHEMA && record.name == self.names[index]).then_some(record)
    }

    /// What the `n`th clean module's kept parts asked of the program, read with its types as found
    /// again (`ember_typeck::Reuse::kept`). The parts this build uses are read here, once; an
    /// instance that cannot be read is made again, as one that does not hold is.
    pub(crate) fn kept(&self, n: usize, found: &Found) -> Result<Kept, String> {
        let clean = &self.clean[n];
        let read = |part: &Part| -> Result<Form, String> {
            ember_span::codec::decode_relocated(clean.files.clone(), false, || {
                ember_types::codec::decode(found.clone(), || ember_span::codec::from_bytes::<Form>(&part.form))
            })
            .map_err(|error| format!("the kept form of `{}` cannot be read: {error}", clean.record.name))
        };
        let used = |part: &Part, form: &Form| Used {
            module: clean.module,
            facts: part.facts.clone(),
            uses: part.uses.clone(),
            kept: form.kept.clone(),
            diagnostics: form.diagnostics.clone(),
            body_diagnostics: form.body_diagnostics.clone(),
        };
        let mut parts: Vec<(Used, Form, Option<String>)> = Vec::new();
        if clean.whole {
            let form = read(&clean.record.module)?;
            parts.push((used(&clean.record.module, &form), form, None));
        }
        for ((body, part), holds) in clean.record.units.iter().zip(&clean.units) {
            if !holds {
                continue;
            }
            match (body, read(part)) {
                (Some(_), form) => {
                    let form = form?;
                    parts.push((used(part, &form), form, None));
                }
                (None, Ok(form)) => parts.push((used(part, &form), form, Some(part.key().to_string()))),
                (None, Err(_)) => {}
            }
        }
        let mut kept = Kept::default();
        for (part, _, instance) in &parts {
            // An instance's requests are its own body's: it is made because another part asks.
            if instance.is_none() {
                kept.requests.extend(part.kept.requests.iter().cloned());
                kept.holders.extend(part.kept.holders.iter().copied());
            }
        }
        self.read.lock().unwrap_or_else(|e| e.into_inner())[n] = Some(parts);
        Ok(kept)
    }

    /// After the check: each clean module whose types were not found again, or whose parts could
    /// not be read, is checked whole by the next attempt. Whether this attempt can go on.
    pub(crate) fn found(&self, checked: &CheckOutput, attempts: &mut Attempts) -> bool {
        let read = self.read.lock().unwrap_or_else(|e| e.into_inner());
        for ((clean, found), parts) in self.clean.iter().zip(&checked.found).zip(read.iter()) {
            if found.is_err() || parts.is_none() {
                attempts.forced.insert(clean.record.name.clone());
                attempts.again = true;
            }
        }
        !attempts.again
    }

    /// The kept parts' bodies, after the program's own (which take the place of any kept body of
    /// the same name), each marked `kept`; and their contracts, which the region summaries take as
    /// given. A generic's instance this build made again takes a kept form of it: the same body,
    /// already through the checks.
    pub(crate) fn inject(&mut self, bodies: &mut Vec<Body>) -> KeptFacts {
        let mut facts = KeptFacts::default();
        // A module checked in part makes its interface artifact again.
        for clean in self.clean.iter().filter(|clean| clean.whole) {
            let (name, dependencies, hash) = &clean.record.interface;
            facts.interfaces.insert(
                name.clone(),
                ember_build::interface::KnownInterface {
                    dependencies: dependencies.clone(),
                    interface_hash: ember_build::interface::InterfaceHash::from_bytes(*hash),
                },
            );
        }
        let parts: Vec<(Used, Form, Option<String>)> = self.read.get_mut().unwrap_or_else(|e| e.into_inner()).iter_mut().flatten().flat_map(std::mem::take).collect();
        let instances: HashSet<String> = parts.iter().filter_map(|(_, _, instance)| instance.clone()).collect();
        bodies.retain(|body| !instances.contains(&body.symbol));
        let mut present: HashSet<String> = bodies.iter().map(|body| body.symbol.clone()).collect();
        for (used, form, instance) in parts {
            if let Some(symbol) = &instance
                && present.contains(symbol)
            {
                continue;
            }
            let mut contracts: HashMap<String, CallRegionContract> = form.contracts.into_iter().collect();
            let mut stores: HashMap<String, StoreSummary> = form.stores.into_iter().collect();
            let invalid: HashSet<String> = form.invalid.into_iter().collect();
            for mut body in form.bodies {
                if instance.is_none() {
                    self.kept_functions.insert(body.symbol.clone());
                }
                if !present.insert(body.symbol.clone()) {
                    continue;
                }
                body.kept = true;
                if let Some(contract) = contracts.remove(&body.symbol) {
                    facts.contracts.insert(body.symbol.clone(), contract);
                }
                if let Some(summary) = stores.remove(&body.symbol) {
                    facts.borrow.stores.insert(body.symbol.clone(), summary);
                }
                if invalid.contains(&body.symbol) {
                    facts.borrow.invalid.insert(body.symbol.clone());
                }
                bodies.push(body);
            }
            let key = used.facts.first().map(|(symbol, _)| symbol.clone()).unwrap_or_default();
            match instance {
                Some(symbol) => {
                    self.used_instances.insert(symbol, used);
                }
                None => {
                    self.used_parts.insert(format!("{}\0{key}", used.module), used);
                }
            }
        }
        facts
    }

    /// The warnings the type check of the kept parts reported, reported again: each one not
    /// reported by this build already.
    pub(crate) fn replay(&mut self, sink: &mut Sink, map: &SourceMap) {
        let diagnostics: Vec<Diagnostic> =
            self.read.get_mut().unwrap_or_else(|e| e.into_inner()).iter_mut().flatten().flatten().flat_map(|(used, _, _)| std::mem::take(&mut used.diagnostics)).collect();
        report_again(diagnostics, sink, map);
    }

    /// The warnings the checks of the kept parts' MIR reported (those this build uses, `inject`),
    /// reported again where a clean build's checks of them report.
    pub(crate) fn replay_bodies(&mut self, sink: &mut Sink, map: &SourceMap) {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();
        for used in self.used_parts.values().chain(self.used_instances.values()) {
            diagnostics.extend(used.body_diagnostics.iter().cloned());
        }
        report_again(diagnostics, sink, map);
    }

    /// After the checks: whether the facts each kept part used are still the facts of the bodies
    /// it calls. A part whose are not is checked again by the next attempt.
    pub(crate) fn verify(&self, checks: &Checks, contracts: &HashMap<String, CallRegionContract>, borrow: &BorrowFacts, attempts: &mut Attempts) -> bool {
        // Only a body made this build can have facts other than its kept ones.
        let mut now: HashMap<String, Hash> = HashMap::new();
        let mut stale = |used: &Used| {
            used.uses.iter().any(|(symbol, then)| {
                checks.fresh.contains(symbol) && *now.entry(symbol.clone()).or_insert_with(|| facts_of(symbol, contracts, borrow)) != *then
            })
        };
        for (key, used) in &self.used_parts {
            if stale(used) {
                let (_, part) = key.split_once('\0').expect("a module and a part");
                attempts.forced_parts.insert((self.names[used.module].clone(), part.to_string()));
                attempts.again = true;
            }
        }
        for (symbol, used) in &self.used_instances {
            if stale(used) {
                attempts.forced_instances.insert(symbol.clone());
                attempts.again = true;
            }
        }
        !attempts.again
    }

    /// A program whose check asked the compile-time evaluator for a value keeps nothing, and drops
    /// what was kept before: a value reads what no source says (a file, a variable, `[CT-4]`), which
    /// no record holds, and the modules that name it would keep the old one.
    pub(crate) fn forget(&self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }

    /// Keep each module checked here, whole or in part, for the next build. `bodies` — the
    /// program's MIR as the checks left it; `contracts` — every body's region contract.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn keep(
        &self,
        checks: &Checks,
        artifacts: &[ember_build::interface::ModuleInterfaceArtifact],
        bodies: &[Body],
        contracts: &HashMap<String, CallRegionContract>,
        borrow: &BorrowFacts,
        sink: &Sink,
        types: &TypeTable,
        made: &(dyn Fn(ember_types::StructId) -> Option<Option<ember_types::codec::MadeCall>> + Sync),
        map: &SourceMap,
    ) -> Result<(), String> {
        let by_symbol: HashMap<&str, &Body> = bodies.iter().map(|body| (body.symbol.as_str(), body)).collect();
        let artifacts: HashMap<&str, &ember_build::interface::ModuleInterfaceArtifact> =
            artifacts.iter().map(|artifact| (artifact.module.as_str(), artifact)).collect();
        let reported = |places: &[usize]| -> Vec<Diagnostic> { places.iter().map(|&at| sink.diagnostics()[at].clone()).collect() };
        std::fs::create_dir_all(&self.dir).map_err(|error| format!("cannot write `{}`: {error}", self.dir.display()))?;
        // `[BLD-7]`, ODR-281 — each module's record is made from what this build read of it alone,
        // and written to a file of its own: the records are made on the `-j` threads.
        let checked: Vec<usize> = (0..checks.modules.len()).filter(|&index| checks.modules[index].is_some()).collect();
        let written = ember_analysis::parallel::each(&checked, &mut Sink::new(), |&index, _| -> Result<(), String> {
            let Some(module) = &checks.modules[index] else { return Ok(()) };
            let Some(artifact) = artifacts.get(checks.interface_names[index].as_str()) else { return Ok(()) };
            let interface = (artifact.module.clone(), artifact.dependencies.keys().cloned().collect(), artifact.interface_hash.bytes());
            // Each part: its symbols, and its form; a kept one's as its record had it.
            let form = |symbols: &[String], kept: Kept, diagnostics: Vec<Diagnostic>, body_diagnostics: Vec<Diagnostic>| Form {
                bodies: symbols
                    .iter()
                    .filter_map(|symbol| by_symbol.get(symbol.as_str()).map(|body| (*body).clone()))
                    .map(|mut body| {
                        body.kept = false;
                        body
                    })
                    .collect(),
                contracts: symbols.iter().filter_map(|symbol| contracts.get(symbol).map(|contract| (symbol.clone(), contract.clone()))).collect(),
                stores: symbols.iter().filter_map(|symbol| borrow.stores.get(symbol).map(|summary| (symbol.clone(), summary.clone()))).collect(),
                invalid: symbols.iter().filter(|symbol| borrow.invalid.contains(*symbol)).cloned().collect(),
                kept,
                diagnostics,
                body_diagnostics,
            };
            let fresh_uses = |symbols: &[String]| -> Vec<(String, Hash)> {
                let own: HashSet<&String> = symbols.iter().collect();
                let mut used = BTreeSet::new();
                for symbol in symbols {
                    if let Some(body) = by_symbol.get(symbol.as_str()) {
                        crate::collect_body_function_symbols(body, &mut used);
                    }
                }
                used.into_iter().filter(|symbol| !own.contains(symbol) && by_symbol.contains_key(symbol.as_str())).map(|symbol| {
                    let facts = facts_of(&symbol, contracts, borrow);
                    (symbol, facts)
                }).collect()
            };
            let fresh_facts = |symbols: &[String]| -> Vec<(String, Hash)> {
                symbols.iter().filter(|symbol| by_symbol.contains_key(symbol.as_str())).map(|symbol| (symbol.clone(), facts_of(symbol, contracts, borrow))).collect()
            };
            // (body piece, symbols, uses, facts, form)
            let mut parts: Vec<(Option<u32>, Vec<(String, Hash)>, Vec<(String, Hash)>, Form)> = Vec::new();
            let module_symbols = module.module_part.functions.clone();
            parts.push((
                None,
                fresh_facts(&module_symbols),
                fresh_uses(&module_symbols),
                form(&module_symbols, module.module_part.kept.clone(), reported(&module.module_part.diagnostics), reported(&module.module_part.body_diagnostics)),
            ));
            for unit in &module.units {
                let part = match &unit.fresh {
                    Some(fresh) => (
                        Some(unit.body),
                        fresh_facts(&unit.functions),
                        fresh_uses(&unit.functions),
                        form(&unit.functions, fresh.kept.clone(), reported(&fresh.diagnostics), reported(&fresh.body_diagnostics)),
                    ),
                    None => {
                        let Some(used) = self.used_parts.get(&format!("{index}\0{}", unit.functions.first().map_or("", String::as_str))) else {
                            continue;
                        };
                        let symbols: Vec<String> = used.facts.iter().map(|(symbol, _)| symbol.clone()).collect();
                        (
                            Some(unit.body),
                            used.facts.clone(),
                            used.uses.clone(),
                            form(&symbols, used.kept.clone(), used.diagnostics.clone(), used.body_diagnostics.clone()),
                        )
                    }
                };
                parts.push(part);
            }
            for symbol in &checks.instances[index] {
                let symbols = std::slice::from_ref(symbol);
                let part = match self.used_instances.get(symbol) {
                    Some(used) => (None, used.facts.clone(), used.uses.clone(), form(symbols, Kept::default(), Vec::new(), used.body_diagnostics.clone())),
                    None => {
                        let diagnostics = reported(checks.instance_diagnostics.get(symbol).map_or(&[][..], Vec::as_slice));
                        (None, fresh_facts(symbols), fresh_uses(symbols), form(symbols, Kept::default(), Vec::new(), diagnostics))
                    }
                };
                parts.push(part);
            }
            // Written with one numbering of files and of types; each part's spans gathered apart.
            let ((written, types_bytes), extents) = ember_span::codec::encode(|| {
                let (written, mut numbering) = ember_types::codec::encode(|| {
                    parts
                        .iter()
                        .map(|(_, _, _, form)| ember_span::codec::scope(|| ember_span::codec::to_bytes(form)))
                        .map(|(bytes, scoped)| bytes.map(|bytes| (bytes, scoped)))
                        .collect::<Result<Vec<_>, _>>()
                });
                let record = ember_types::codec::record(types, &mut numbering, made);
                (written, record.map(|record| ember_span::codec::to_bytes(&record)))
            });
            let (Ok(written), Ok(Ok(types_bytes))) = (written, types_bytes) else { return Ok(()) };
            let ((files, at), pieces_of) = self.file_refs(extents, map);
            let mut records = parts.into_iter().zip(written).map(|((body, facts, uses, _), (bytes, scoped))| {
                let mut pieces: Vec<(u32, u32)> =
                    scoped.iter().flat_map(|(file, ranges)| pieces_of(*file, ranges).into_iter().map(|piece| (*file, at[*file as usize][&piece]))).collect();
                pieces.sort_unstable();
                pieces.dedup();
                (body, Part { pieces, facts, uses, form: bytes })
            });
            let (_, module_part) = records.next().expect("the module part first");
            let units: Vec<(Option<u32>, Part)> = records.collect();
            let record = Record {
                schema: SCHEMA,
                name: self.names[index].clone(),
                key: self.key,
                source: hash(map.file(checks.files[index]).text.as_bytes()),
                declarations: self.shapes[index].declarations,
                deps: self.imports[index].iter().map(|&dep| (self.names[dep].clone(), self.reach[dep])).collect(),
                files,
                types: types_bytes,
                module: module_part,
                units,
                interface,
            };
            let bytes = ember_span::codec::to_bytes(&record).map_err(|error| error.to_string())?;
            // Written aside, then renamed into place: a build reading it sees the old record or the new.
            let path = self.path(index);
            let partial = path.with_extension(format!("{}.partial", std::process::id()));
            std::fs::write(&partial, bytes).map_err(|error| format!("cannot write `{}`: {error}", partial.display()))?;
            if std::fs::rename(&partial, &path).is_err() {
                let _ = std::fs::remove_file(&partial);
            }
            Ok(())
        });
        written.into_iter().try_for_each(|(result, _)| result)
    }

    /// The files a record's spans point into (`ember_span::codec::encode`'s extents), as the record
    /// keeps them; for each, the place in its `pieces` of each piece of the file (by its place
    /// among the file's pieces); and a function giving the pieces of a file (by number) a part's
    /// ranges lie in.
    #[allow(clippy::type_complexity)]
    fn file_refs<'s>(
        &'s self,
        extents: Vec<ember_span::codec::Extent>,
        map: &'s SourceMap,
    ) -> ((Vec<FileRef>, Vec<HashMap<u32, u32>>), impl Fn(u32, &[(u32, u32)]) -> Vec<u32> + 's) {
        let tilings: Vec<Option<Vec<(u32, u32)>>> = extents
            .iter()
            .map(|extent| {
                let module = *self.module_of_file.get(&extent.file)?;
                Some(self.shapes[module].pieces(map.file(extent.file).text.len() as u32))
            })
            .collect();
        let touched = move |tiling: &[(u32, u32)], ranges: &[(u32, u32)]| -> Vec<u32> {
            let mut out = BTreeSet::new();
            for &(start, end) in ranges {
                let first = tiling.partition_point(|&(_, piece_end)| piece_end <= start).min(tiling.len().saturating_sub(1));
                for (index, &(piece_start, _)) in tiling.iter().enumerate().skip(first) {
                    if piece_start >= end.max(start + 1) && index > first {
                        break;
                    }
                    out.insert(index as u32);
                }
            }
            out.into_iter().collect()
        };
        let mut files = Vec::new();
        let mut at = Vec::new();
        for (extent, tiling) in extents.iter().zip(&tilings) {
            let source = map.file(extent.file);
            let path = source.path.to_string_lossy().into_owned();
            match tiling {
                Some(tiling) => {
                    let module = self.module_of_file[&extent.file];
                    let mut places = HashMap::new();
                    let pieces: Vec<Piece> = touched(tiling, &extent.ranges)
                        .into_iter()
                        .enumerate()
                        .map(|(place, index)| {
                            places.insert(index, place as u32);
                            let (start, end) = tiling[index as usize];
                            let body = (index % 2 == 1).then(|| hash(&source.text.as_bytes()[start as usize..end as usize]));
                            Piece { index, start, length: end - start, body }
                        })
                        .collect();
                    files.push(FileRef { path, kind: FileKind::Module { declarations: self.shapes[module].declarations, pieces } });
                    at.push(places);
                }
                None => {
                    files.push(FileRef { path, kind: FileKind::Whole(hash(source.text.as_bytes())) });
                    at.push(HashMap::new());
                }
            }
        }
        let pieces_of = move |file: u32, ranges: &[(u32, u32)]| match &tilings[file as usize] {
            Some(tiling) => touched(tiling, ranges),
            None => Vec::new(),
        };
        ((files, at), pieces_of)
    }
}

/// A body's facts (`[BLD-2]`): its region contract, what it stores where its callers' regions do
/// not reach, and whether its result can be followed, hashed. These name parameters and paths into
/// them, never a type or a place.
fn facts_of(symbol: &str, contracts: &HashMap<String, CallRegionContract>, borrow: &BorrowFacts) -> Hash {
    let facts = (contracts.get(symbol), borrow.stores.get(symbol), borrow.invalid.contains(symbol));
    hash(&ember_span::codec::to_bytes(&facts).expect("facts are written"))
}

/// What a build learned of each module it checked, whole or in part, for `Plan::keep` and
/// `Plan::verify`.
pub(crate) struct Checks {
    /// Each module's file, and its name among the interface artifacts.
    pub files: Vec<FileId>,
    pub interface_names: Vec<String>,
    /// Each module checked, where all its bodies asked of the program can be asked again.
    pub modules: Vec<Option<ModuleChecked>>,
    /// The generics' instances each checked module's bodies reach.
    pub instances: Vec<Vec<String>>,
    /// The places in the sink of the warnings each instance's checks reported.
    pub instance_diagnostics: HashMap<String, Vec<usize>>,
    /// The bodies this build checked, by symbol.
    pub fresh: HashSet<String>,
}

/// A module this build checked: what its check made outside its functions' parts, and each of
/// those.
pub(crate) struct ModuleChecked {
    pub module_part: Checked,
    pub units: Vec<UnitChecked>,
}

/// One part as this build checked it: its bodies, what they asked, and where in the sink the
/// type check and the MIR checks of them reported.
pub(crate) struct Checked {
    pub functions: Vec<String>,
    pub kept: Kept,
    pub diagnostics: Vec<usize>,
    pub body_diagnostics: Vec<usize>,
}

/// A function with a body of its own: its body's place among its file's pieces, its bodies' symbols
/// (a kept one's: its own function's), and, checked here, what its check found.
pub(crate) struct UnitChecked {
    pub body: u32,
    pub functions: Vec<String>,
    pub fresh: Option<Checked>,
}

impl Checks {
    /// From the type check's account of each module (`checked.modules`), the kept bodies that are
    /// not instances (`Plan::kept_functions`), the program's bodies as the checks left them, and the
    /// sink ranges each body check reported in (`mir_ranges`: the borrow checks' and the lints', by
    /// body).
    pub(crate) fn new(
        plan: &Plan,
        checked: &CheckOutput,
        modules: &[LoadedModule],
        bodies: &[Body],
        mir_ranges: &[(usize, Range<usize>)],
        sink: &Sink,
    ) -> Checks {
        let files: Vec<FileId> = modules.iter().map(|loaded| loaded.module.span.file).collect();
        let module_of_file: HashMap<FileId, usize> = files.iter().enumerate().map(|(index, file)| (*file, index)).collect();
        let present: HashSet<&str> = bodies.iter().map(|body| body.symbol.as_str()).collect();
        let warning = |at: &usize| matches!(sink.diagnostics()[*at].severity, Severity::Warning | Severity::Lint);
        let mut body_ranges: HashMap<&str, Vec<usize>> = HashMap::new();
        for (body, range) in mir_ranges {
            body_ranges.entry(bodies[*body].symbol.as_str()).or_default().extend(range.clone().filter(warning));
        }
        let body_diagnostics = |symbols: &[String]| -> Vec<usize> {
            symbols.iter().filter_map(|symbol| body_ranges.get(symbol.as_str())).flatten().copied().collect()
        };
        // `L3010` comes after every body, for the block it names: its body's part's.
        let mut unneeded: HashMap<usize, Vec<(Span, usize)>> = HashMap::new();
        for at in checked.unneeded_unsafe.clone().filter(warning) {
            let span = sink.diagnostics()[at].primary.span;
            if let Some(&module) = module_of_file.get(&span.file) {
                unneeded.entry(module).or_default().push((span, at));
            }
        }
        let mut fresh: HashSet<String> = HashSet::new();
        let checked_modules: Vec<Option<ModuleChecked>> = checked
            .modules
            .iter()
            .enumerate()
            .map(|(index, check)| {
                let check = check.as_ref()?;
                let kept = check.kept.clone()?;
                let slice = |requests: &Range<usize>, holders: &Range<usize>| Kept {
                    requests: kept.requests[requests.clone()].to_vec(),
                    holders: kept.holders[holders.clone()].to_vec(),
                };
                let shape = &plan.shapes[index];
                let body_piece = |span: Span| shape.bodies.binary_search(&(span.start, span.end)).ok().map(|body| body as u32 * 2 + 1);
                let mut in_parts: HashSet<&String> = HashSet::new();
                let mut covered: Vec<Range<usize>> = Vec::new();
                let mut requests_in: Vec<Range<usize>> = Vec::new();
                let mut holders_in: Vec<Range<usize>> = Vec::new();
                let mut units = Vec::new();
                for part in &check.parts {
                    let Some(body) = body_piece(part.body) else { continue };
                    let functions: Vec<String> = part.functions.iter().filter(|symbol| present.contains(symbol.as_str())).cloned().collect();
                    in_parts.extend(part.functions.iter());
                    covered.push(part.diagnostics.clone());
                    requests_in.push(part.requests.clone());
                    holders_in.push(part.holders.clone());
                    let fresh_part = (!part.kept).then(|| {
                        fresh.extend(functions.iter().cloned());
                        let mut diagnostics: Vec<usize> = part.diagnostics.clone().filter(warning).collect();
                        diagnostics.extend(unneeded.get(&index).into_iter().flatten().filter(|(span, _)| span.start >= part.body.start && span.end <= part.body.end).map(|(_, at)| *at));
                        Checked {
                            functions: functions.clone(),
                            kept: slice(&part.requests, &part.holders),
                            diagnostics,
                            body_diagnostics: body_diagnostics(&functions),
                        }
                    });
                    units.push(UnitChecked { body, functions, fresh: fresh_part });
                }
                let outside = |at: &usize, ranges: &[Range<usize>]| !ranges.iter().any(|range| range.contains(at));
                let module_functions: Vec<String> =
                    check.functions.iter().filter(|symbol| !in_parts.contains(symbol) && present.contains(symbol.as_str())).cloned().collect();
                fresh.extend(module_functions.iter().cloned());
                let mut diagnostics: Vec<usize> = check.diagnostics.clone().filter(warning).filter(|at| outside(at, &covered)).collect();
                let unit_bodies: Vec<Span> = check.parts.iter().filter(|part| body_piece(part.body).is_some()).map(|part| part.body).collect();
                diagnostics.extend(
                    unneeded
                        .get(&index)
                        .into_iter()
                        .flatten()
                        .filter(|(span, _)| !unit_bodies.iter().any(|body| span.start >= body.start && span.end <= body.end))
                        .map(|(_, at)| *at),
                );
                let outside_all = |ranges: &[Range<usize>], len: usize| (0..len).filter(|at| outside(at, ranges)).collect::<Vec<usize>>();
                let module_kept = Kept {
                    requests: outside_all(&requests_in, kept.requests.len()).into_iter().map(|at| kept.requests[at].clone()).collect(),
                    holders: outside_all(&holders_in, kept.holders.len()).into_iter().map(|at| kept.holders[at]).collect(),
                };
                let module_part = Checked {
                    body_diagnostics: body_diagnostics(&module_functions),
                    functions: module_functions,
                    kept: module_kept,
                    diagnostics,
                };
                Some(ModuleChecked { module_part, units })
            })
            .collect();
        // The instances each checked module's bodies reach: the program's bodies no module made,
        // through calls and the other ways one body names another.
        let made: HashSet<&str> = checked
            .modules
            .iter()
            .flatten()
            .flat_map(|check| check.functions.iter().map(String::as_str))
            .chain(plan.kept_functions.iter().map(String::as_str))
            .collect();
        let by_symbol: HashMap<&str, &Body> = bodies.iter().map(|body| (body.symbol.as_str(), body)).collect();
        let instances: Vec<Vec<String>> = checked_modules
            .iter()
            .enumerate()
            .map(|(index, module)| {
                let Some(module) = module else { return Vec::new() };
                let mut reached: BTreeSet<String> = BTreeSet::new();
                let own = checked.modules[index].as_ref().map_or(&[][..], |check| check.functions.as_slice());
                let mut pending: Vec<&str> =
                    own.iter().map(String::as_str).chain(module.units.iter().flat_map(|unit| unit.functions.iter().map(String::as_str))).collect();
                while let Some(symbol) = pending.pop() {
                    let Some(body) = by_symbol.get(symbol) else { continue };
                    let mut referenced = BTreeSet::new();
                    crate::collect_body_function_symbols(body, &mut referenced);
                    for target in referenced {
                        if !made.contains(target.as_str()) && by_symbol.contains_key(target.as_str()) && reached.insert(target.clone()) {
                            pending.push(by_symbol[target.as_str()].symbol.as_str());
                        }
                    }
                }
                reached.into_iter().collect()
            })
            .collect();
        let mut instance_diagnostics: HashMap<String, Vec<usize>> = HashMap::new();
        for symbol in instances.iter().flatten() {
            if let Some(places) = body_ranges.get(symbol.as_str()) {
                instance_diagnostics.insert(symbol.clone(), places.clone());
            }
        }
        // Every body this build made rather than kept: its facts are this build's to give.
        fresh.extend(bodies.iter().filter(|body| !body.kept).map(|body| body.symbol.clone()));
        let interface_names = modules.iter().map(|loaded| crate::module_identity(&loaded.path)).collect();
        Checks { files, interface_names, modules: checked_modules, instances, instance_diagnostics, fresh }
    }
}

/// Report `diagnostics` again, each one this build has not reported (an instance kept in several
/// records reports once, as its one check in a clean build does).
fn report_again(diagnostics: Vec<Diagnostic>, sink: &mut Sink, map: &SourceMap) {
    let mut reported: HashSet<String> = sink.diagnostics().iter().map(|diagnostic| ember_diag::to_json(diagnostic, map).to_string()).collect();
    for diagnostic in diagnostics {
        if reported.insert(ember_diag::to_json(&diagnostic, map).to_string()) {
            sink.emit(diagnostic);
        }
    }
}

/// `[BLD-2]` — the declarations every module sees without importing anything: each `extend` (an
/// implementation is the program's) and each class (a subclass is the program's, which a virtual
/// call and a handle's drop reach), by module, as one hash. Part of every kept form's key, and of
/// every module's interface-cache key.
pub(crate) fn open_world(modules: &[LoadedModule], map: &SourceMap) -> Hash {
    let names: Vec<String> = modules.iter().map(|loaded| module_name(loaded, map)).collect();
    let open: Vec<String> = modules.iter().map(|loaded| declarations(loaded, map).open).collect();
    open_world_of(&names, &open)
}

fn open_world_of(names: &[String], open: &[String]) -> Hash {
    let mut order: Vec<usize> = (0..names.len()).collect();
    order.sort_by(|a, b| names[*a].cmp(&names[*b]));
    let mut hasher = blake3::Hasher::new();
    for index in order {
        for text in [&names[index], &open[index]] {
            hasher.update(&(text.len() as u64).to_le_bytes());
            hasher.update(text.as_bytes());
        }
    }
    *hasher.finalize().as_bytes()
}

/// A module's name in the build: its file's path as it was read.
fn module_name(loaded: &LoadedModule, map: &SourceMap) -> String {
    map.file(loaded.module.span.file).path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// What a module declares, apart from its ordinary functions' bodies.
struct Shape {
    /// Its source without the bodies of its ordinary functions and methods (no generics of their
    /// own or of their type, not a generator, not returning `some I`), which no importer reads,
    /// hashed.
    declarations: Hash,
    /// Its declarations every module sees without importing it, spelled in the same way: each
    /// `extend` and each class.
    open: String,
    /// Those bodies (their blocks' spans), sorted; and where each body's piece of the file ends:
    /// its block's end, and after it the rest of its line and the blank and comment lines up to the
    /// next line of code (an edit there is the body's, not the declarations').
    bodies: Vec<(u32, u32)>,
    ends: Vec<u32>,
}

impl Shape {
    /// The file's pieces, in order, covering it: the text before the first body, the body, the
    /// text after it, …, the text after the last (a body's place is odd).
    fn pieces(&self, length: u32) -> Vec<(u32, u32)> {
        let mut out = Vec::with_capacity(self.bodies.len() * 2 + 1);
        let mut at = 0;
        for (&(start, _), &end) in self.bodies.iter().zip(&self.ends) {
            out.push((at, start));
            out.push((start, end));
            at = end;
        }
        out.push((at, length.max(at)));
        out
    }

    /// Piece `index`, a body's: its block's span now.
    fn piece(&self, index: u32) -> (u32, u32) {
        self.bodies[index as usize / 2]
    }
}

/// Where a body's piece of `text` ends, its block ending at `end` (`Shape::ends`).
fn piece_end(text: &[u8], end: usize) -> usize {
    let skippable = |line: &[u8]| {
        let trimmed = line.trim_ascii_start();
        trimmed.is_empty() || (trimmed.starts_with(b"#") && !trimmed.starts_with(b"##"))
    };
    let line_end = |at: usize| text[at..].iter().position(|&byte| byte == b'\n').map_or(text.len(), |n| at + n + 1);
    let first = line_end(end);
    if !skippable(&text[end..first]) {
        return end;
    }
    let mut at = first;
    while at < text.len() {
        let next = line_end(at);
        if !skippable(&text[at..next]) {
            break;
        }
        at = next;
    }
    at
}

fn declarations(loaded: &LoadedModule, map: &SourceMap) -> Shape {
    use ember_ast::{ItemKind, MemberKind};
    let source = map.file(loaded.module.span.file);
    let text = source.text.as_str();
    let mut bodies: Vec<Span> = Vec::new();
    let ordinary = |decl: &ember_ast::FnDecl| {
        decl.generics.is_empty()
            && !decl.is_gen
            && decl.ret.as_ref().is_none_or(|ret| !map.snippet(ret.span).split(|c: char| !c.is_alphanumeric() && c != '_').any(|word| word == "some"))
    };
    let members = |owner_generic: bool, members: &[ember_ast::Member], bodies: &mut Vec<Span>| {
        for member in members {
            if let MemberKind::Fn(decl) = &member.kind
                && !owner_generic
                && ordinary(decl)
                && let Some(body) = &decl.body
            {
                bodies.push(body.span);
            }
        }
    };
    let mut open = Vec::new();
    for item in &loaded.module.items {
        if item.span.file != source.id {
            continue;
        }
        match &item.kind {
            ItemKind::Fn(decl) if ordinary(decl) => bodies.extend(decl.body.as_ref().map(|body| body.span)),
            ItemKind::Struct(decl) => members(!decl.generics.is_empty(), &decl.members, &mut bodies),
            ItemKind::Enum(decl) => members(!decl.generics.is_empty(), &decl.members, &mut bodies),
            ItemKind::Class(decl) => {
                members(!decl.generics.is_empty(), &decl.members, &mut bodies);
                open.push(item.span);
            }
            ItemKind::Extend(decl) => {
                members(!decl.generics.is_empty(), &decl.members, &mut bodies);
                open.push(item.span);
            }
            _ => {}
        }
    }
    bodies.retain(|body| body.file == source.id);
    bodies.sort_by_key(|body| body.start);
    // Bodies are apart; one inside another (none is) would be the outer's.
    bodies.dedup_by(|inner, outer| inner.start >= outer.start && inner.end <= outer.end);
    let ends: Vec<u32> = bodies
        .iter()
        .enumerate()
        .map(|(index, body)| {
            let limit = bodies.get(index + 1).map_or(text.len(), |next| next.start as usize);
            piece_end(text.as_bytes(), body.end as usize).min(limit) as u32
        })
        .collect();
    let without = |range: Span| -> String {
        let mut out = String::new();
        let mut at = range.start as usize;
        for (body, &end) in bodies.iter().zip(&ends).filter(|(body, _)| body.start >= range.start && body.end <= range.end) {
            if (body.start as usize) < at {
                continue;
            }
            out.push_str(&text[at..body.start as usize]);
            out.push('\u{1}');
            at = (end as usize).min(range.end as usize).max(body.end as usize);
        }
        out.push_str(&text[at..range.end as usize]);
        out
    };
    let whole = without(Span::new(source.id, 0, text.len() as u32));
    let open: String = open.into_iter().map(without).collect::<Vec<_>>().join("\0");
    Shape { declarations: hash(whole.as_bytes()), open, bodies: bodies.iter().map(|body| (body.start, body.end)).collect(), ends }
}

/// Each module's imports, by their places in `modules`. The prelude's modules are every module's
/// (`[MOD-5]`).
fn direct_imports(modules: &[LoadedModule]) -> Vec<Vec<usize>> {
    let by_path: HashMap<String, usize> = modules.iter().enumerate().map(|(index, loaded)| (loaded.path.join("."), index)).collect();
    let std = ember_branding::STD_PACKAGE;
    let prelude: Vec<usize> =
        ["core", "collections", "mem", "string"].iter().filter_map(|module| by_path.get(&format!("{std}.{module}")).copied()).collect();
    modules
        .iter()
        .enumerate()
        .map(|(index, loaded)| {
            let mut out: BTreeSet<usize> = prelude.iter().copied().filter(|&module| module != index).collect();
            for import in &loaded.module.imports {
                let path = match &import.kind {
                    ember_ast::ImportKind::Module { path, .. } | ember_ast::ImportKind::Items { path, .. } => path,
                    ember_ast::ImportKind::Foreign { .. } => continue,
                };
                let path: Vec<String> = path.iter().map(|segment| segment.name.to_string()).collect();
                let joined = path.join(".");
                let found = by_path.get(&joined).or_else(|| by_path.get(&format!("{std}.{joined}")));
                // `a.b.c`'s module, or a module above an item it names (`from a.b import c`).
                let found = found.copied().or_else(|| {
                    (1..path.len()).rev().find_map(|end| by_path.get(&path[..end].join(".")).copied())
                });
                out.extend(found.filter(|&module| module != index));
            }
            // `[RFL-2]`, `[SER-1]` — `std.ser`, loaded wherever its derives are written.
            if let Some(&ser) = by_path.get(&format!("{std}.ser"))
                && ser != index
                && loaded.module.items.iter().any(|item| item.attrs.iter().any(|attr| {
                    attr.path.len() == 1 && (attr.path[0].name.is("reflect") || attr.path[0].name.is("derive"))
                }))
            {
                out.insert(ser);
            }
            out.into_iter().collect()
        })
        .collect()
}
