//! `[EFF-24]` (A4-2, the first slice) — what each function may change, read and end, and through which
//! chain of calls, worked out from its code over the whole program's calls (`targets.rs`).
//!
//! A fact names a place a parameter reaches (`this` is a method's first): the parameter, then the
//! steps from it, at least one of them through a reference or an object's handle, since a change of
//! a parameter's own copy is no change its caller can see. A fact covers everything inside its
//! place. A body's own lines give its facts; a call gives its callee's, mapped onto the arguments,
//! for a call through a function value or a `dyn` or `virtual` method those of every function that
//! may run there (`[EFF-2]`, `[DSP-5]`, `[CLS-4]`); a `drop` ends what it drops. Whatever the
//! analysis cannot follow (a pointer read through a raw pointer, the result of a function it has no
//! body of) may be anything a parameter reaches, so nothing is missed: a fact too many refuses a
//! correct program later, a fact missing would accept a wrong one. Paths are cut at `MAX_PATH`
//! steps, a shorter path covering the longer ones, so recursion through a chain of links ends.
//!
//! Nothing is refused by it yet (M2): its first consumer is M1's `this`, whose change of its object
//! is worked out from its body (`[FN-4]`, `[EXC-15]`). `ember build --emit mir` prints it.

use std::collections::{HashMap, HashSet};

use ember_mir::{Body, Builtin, Const, FuncRef, LocalId, Operand, Place, Projection, Rvalue, SoAOp, StmtKind, Terminator};
use ember_span::Span;
use ember_types::{Ty, TyKind, TypeTable};

use crate::regions::{place_type, project_type, Origin, Point, Regions};
use crate::drop_runs::DropRuns;
use crate::targets::Targets;

/// A path longer than this is cut to its first `MAX_PATH` steps, which cover it.
const MAX_PATH: usize = 8;

/// One step from a place to a place inside it, or to what it points at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Step {
    Field(usize),
    /// Any element of a list or an array.
    Elements,
    Variant(usize),
    Column(usize),
    /// Through a reference, a pointer or an object's handle.
    Deref,
}

/// Where a fact's place starts: a parameter (0-based; a method's `this` is 0), or a `static`, the
/// same place in every function (SD1's `Static(symbol, path)`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Root {
    Param(usize),
    /// The static's name and the type of its value.
    Static(String, ember_types::Ty),
}

/// A place a parameter reaches, or a place inside a static.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Reach {
    pub root: Root,
    pub path: Vec<Step>,
}

/// A fact and the chain of calls that makes it: each call's callee (a body's index) and the call's
/// span; empty for the function's own line.
#[derive(Clone, Debug)]
pub struct Fact {
    pub reach: Reach,
    pub chain: Vec<(usize, Span)>,
    /// The line that does it, in the last function of the chain (the function's own line when the
    /// chain is empty): what a message names ("through `note`, line 4").
    pub at: Span,
    /// `[DRP-8]` — what that line does, when it is a built-in a message names ("pushes to").
    pub op: Option<&'static str>,
}

/// `[EFF-24]` — what a function may change, read, reshape and end.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub changes: Vec<Fact>,
    pub reads: Vec<Fact>,
    /// A change that may move or free memory (§VII.1's **reshape**): replacing a value that owns
    /// memory or changing an enum's variant, a list's or a string's elements added, removed,
    /// reordered or replaced. Each is a change too.
    pub reshapes: Vec<Fact>,
    /// A list or string it adds to the end of (`push`, `extend`): each is a reshape too.
    pub adds: Vec<Fact>,
    /// Every other reshape: elements removed, reordered or replaced, or the value replaced, which
    /// may move or free what is already there.
    pub removes: Vec<Fact>,
    pub ends: Vec<Fact>,
    /// `[BRW-21]` — for each view parameter, by index (`this` included), what the function may reshape
    /// or end at a line after which it still uses that parameter or a view made from it, through its
    /// own lines and the same fact of the functions it calls. A change made through the parameter
    /// itself is not one: a `ref mut` name is alone (`[BRW-1]`), and its view of the place holds.
    pub used_after: Vec<(usize, Fact)>,
    /// `[BRW-1]`, `[BRW-21]` — for each `ref mut` parameter, by index, what the function reads and
    /// changes by other names while it still uses that parameter (a static's place among them, which
    /// a call is checked against), through its own lines and the same fact of the functions it calls.
    pub read_while: Vec<(usize, Fact)>,
    pub changed_while: Vec<(usize, Fact)>,
    /// `[CLS-2]` — a method's ways of letting `this` out, which `init` may call only once every field is
    /// set: `this` as a whole (or a copy or view of all of it) passed, stored, captured or returned, or
    /// a method called on it that lets it out. The fields of `this` it reads are its reads of `this`.
    pub lets_this_out: Vec<Fact>,
}

impl Summary {
    /// Whether it may change the place `param`'s `path` reaches, or anything inside it or holding it.
    pub fn may_change(&self, param: usize, path: &[Step]) -> bool {
        self.changes.iter().any(|fact| fact.reach.root == Root::Param(param) && overlaps(&fact.reach.path, path))
    }

    /// Whether it may change anything parameter `param` reaches: for a method's `this`, whether it
    /// changes its object.
    pub fn changes_param(&self, param: usize) -> bool {
        self.changes.iter().any(|fact| fact.reach.root == Root::Param(param))
    }

    /// Whether it may move or free memory inside, or holding, the place `param`'s `path` reaches.
    pub fn may_reshape(&self, param: usize, path: &[Step]) -> bool {
        self.reshapes.iter().any(|fact| fact.reach.root == Root::Param(param) && overlaps(&fact.reach.path, path))
    }

    /// `[EFF-24]` — whether all it does to the list at `reach` is add to its end (`push`, `extend`),
    /// reading or writing no element already in it: it adds there, every reshape there is an add,
    /// and nothing reads, changes or ends an element of it, or ends or reshapes what holds it.
    pub fn only_adds(&self, reach: &Reach) -> bool {
        let at = |fact: &Fact| fact.reach.root == reach.root;
        let element = |fact: &Fact| at(fact) && fact.reach.path.len() > reach.path.len() && fact.reach.path.starts_with(&reach.path) && fact.reach.path[reach.path.len()] == Step::Elements;
        self.adds.iter().any(|fact| fact.reach == *reach)
            && !self.removes.iter().any(|fact| at(fact) && overlaps(&fact.reach.path, &reach.path))
            && !self.reads.iter().chain(&self.changes).chain(&self.ends).any(element)
            && !self.ends.iter().any(|fact| at(fact) && reach.path.starts_with(&fact.reach.path))
    }
}

/// Whether one path lies inside the other.
fn overlaps(a: &[Step], b: &[Step]) -> bool {
    a.iter().zip(b).all(|(x, y)| x == y || matches!((x, y), (Step::Elements, _) | (_, Step::Elements)))
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum What {
    Change,
    Read,
    Reshape,
    Add,
    Remove,
    End,
    /// A reshape or an end after which the view parameter of this index is still used.
    UsedAfter(usize),
    /// A read, or a change, while the `ref mut` parameter of this index is still used.
    ReadWhile(usize),
    ChangedWhile(usize),
    /// `this` let out.
    LetsThisOut,
}

/// Where a place is, in a body's own terms: a local and the steps from it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Spot {
    local: LocalId,
    steps: Vec<Step>,
}

/// What a local holding a pointer may point at.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Target {
    /// `*local` is this place.
    At(Spot),
    /// `*local` is this static's value.
    Static(String, ember_types::Ty),
    /// Not followed: anything any parameter reaches.
    Anything,
}

/// `[EFF-24]` — every body's summary, by index.
pub fn summarize(bodies: &[Body], types: &TypeTable) -> Vec<Summary> {
    let targets = Targets::new(bodies, types);
    let index: HashMap<&str, usize> = bodies.iter().enumerate().map(|(at, body)| (body.symbol.as_str(), at)).collect();
    let pointers: Vec<HashMap<LocalId, HashSet<Target>>> = bodies.iter().map(|body| pointers_of(body, types)).collect();
    // `[BRW-21]` — where each view parameter is still used: the borrow check's own regions.
    let regions = crate::borrows::regions_all(bodies, types);
    let drop_runs = DropRuns::new(bodies, types, &index);
    let mut summaries: Vec<Summary> = vec![Summary::default(); bodies.len()];
    let mut known: Vec<HashSet<(What, Reach)>> = vec![HashSet::new(); bodies.len()];
    // Each round reads what the last one found; facts only grow and are finitely many (paths are cut),
    // so the rounds end.
    loop {
        let mut grew = false;
        for (at, body) in bodies.iter().enumerate() {
            let found = body_facts(at, body, bodies, types, &index, &targets, &pointers[at], regions[at].as_ref(), &summaries, &drop_runs);
            for (what, fact) in found {
                if known[at].insert((what, fact.reach.clone())) {
                    grew = true;
                    let summary = &mut summaries[at];
                    match what {
                        What::Change => summary.changes.push(fact),
                        What::Read => summary.reads.push(fact),
                        What::Reshape => summary.reshapes.push(fact),
                        What::Add => summary.adds.push(fact),
                        What::Remove => summary.removes.push(fact),
                        What::End => summary.ends.push(fact),
                        What::UsedAfter(param) => summary.used_after.push((param, fact)),
                        What::ReadWhile(param) => summary.read_while.push((param, fact)),
                        What::ChangedWhile(param) => summary.changed_while.push((param, fact)),
                        What::LetsThisOut => summary.lets_this_out.push(fact),
                    }
                }
            }
        }
        if !grew {
            break;
        }
    }
    for summary in &mut summaries {
        for list in [&mut summary.changes, &mut summary.reads, &mut summary.reshapes, &mut summary.adds, &mut summary.removes, &mut summary.ends] {
            list.sort_by(|a, b| a.reach.cmp(&b.reach));
        }
        for list in [&mut summary.used_after, &mut summary.read_while, &mut summary.changed_while] {
            list.sort_by(|a, b| (a.0, &a.1.reach).cmp(&(b.0, &b.1.reach)));
        }
    }
    summaries
}

/// `[EFF-24]` (M6a step 1) — every body's summary from its own code and its callees', with no region
/// facts (`used_after`, `read_while`, `changed_while` stay empty: they need the borrow check's regions):
/// what the checks of a build read. Each body is walked once, and again only when a callee's summary
/// grew (a worklist over the callers), not every body every round as [`summarize`] does.
pub fn summarize_flow(bodies: &[Body], types: &TypeTable) -> Vec<Summary> {
    summarize_flow_with(bodies, types, &Targets::new(bodies, types))
}

/// [`summarize_flow`] with the program's call targets already worked out.
pub(crate) fn summarize_flow_with(bodies: &[Body], types: &TypeTable, targets: &Targets) -> Vec<Summary> {
    let targets = targets;
    let index: HashMap<&str, usize> = bodies.iter().enumerate().map(|(at, body)| (body.symbol.as_str(), at)).collect();
    let pointers: Vec<HashMap<LocalId, HashSet<Target>>> = bodies.iter().map(|body| pointers_of(body, types)).collect();
    let drop_runs = DropRuns::new(bodies, types, &index);
    // Who calls whom, as `body_facts` resolves its calls: a body is walked again when one grew.
    let mut callers: Vec<Vec<usize>> = vec![Vec::new(); bodies.len()];
    for (at, body) in bodies.iter().enumerate() {
        for callee in callees_of(at, body, bodies, types, &index, &targets, &drop_runs) {
            if !callers[callee].contains(&at) {
                callers[callee].push(at);
            }
        }
    }
    let mut summaries: Vec<Summary> = vec![Summary::default(); bodies.len()];
    let mut known: Vec<HashSet<(What, Reach)>> = vec![HashSet::new(); bodies.len()];
    let mut queued = vec![true; bodies.len()];
    let mut work: std::collections::VecDeque<usize> = (0..bodies.len()).rev().collect();
    while let Some(at) = work.pop_front() {
        queued[at] = false;
        let found = body_facts(at, &bodies[at], bodies, types, &index, &targets, &pointers[at], None, &summaries, &drop_runs);
        let mut grew = false;
        for (what, fact) in found {
            if known[at].insert((what, fact.reach.clone())) {
                grew = true;
                let summary = &mut summaries[at];
                match what {
                    What::Change => summary.changes.push(fact),
                    What::Read => summary.reads.push(fact),
                    What::Reshape => summary.reshapes.push(fact),
                    What::Add => summary.adds.push(fact),
                    What::Remove => summary.removes.push(fact),
                    What::End => summary.ends.push(fact),
                    What::LetsThisOut => summary.lets_this_out.push(fact),
                    What::UsedAfter(_) | What::ReadWhile(_) | What::ChangedWhile(_) => {}
                }
            }
        }
        if grew {
            for &caller in &callers[at] {
                if !queued[caller] {
                    queued[caller] = true;
                    work.push_back(caller);
                }
            }
        }
    }
    for summary in &mut summaries {
        for list in [&mut summary.changes, &mut summary.reads, &mut summary.reshapes, &mut summary.adds, &mut summary.removes, &mut summary.ends] {
            list.sort_by(|a, b| a.reach.cmp(&b.reach));
        }
    }
    summaries
}

/// `[THR-20]`, `[STA-1]` — `E7012`: a function that may run on another thread (given to `thread.spawn`, or a
/// job submitted outside a scope) and reads or changes a `static mut`, which only the main thread and the
/// tasks and jobs of the scopes it opens reach. One error at each such call, naming the function that does
/// it, by its summary's chain, and the module variables it reaches.
// ponytail: no `guarded by` yet (M9): every module variable is unguarded. Callbacks C may run on another
// thread and `@unsafe(export(..., threads=...))` functions are the C interface's (M12).
pub(crate) fn check_module_variables_on_threads(bodies: &[Body], types: &TypeTable, summaries: &[Summary], targets: &Targets, sink: &mut ember_diag::Sink) {
    let index: HashMap<&str, usize> = bodies.iter().enumerate().map(|(at, body)| (body.symbol.as_str(), at)).collect();
    let drop_runs = DropRuns::new(bodies, types, &index);
    for (at, body) in bodies.iter().enumerate() {
        for block in &body.blocks {
            let Terminator::Call { func: FuncRef::Builtin { which: which @ (Builtin::ThreadSpawn | Builtin::JobSubmit { .. }), .. }, args, .. } = &block.terminator else { continue };
            let runner = if matches!(which, Builtin::JobSubmit { .. }) { "job" } else { "thread" };
            let mut runs: Vec<usize> = Vec::new();
            for operand in args {
                match operand {
                    Operand::Const(Const::Fn(symbol)) => runs.extend(index.get(symbol.as_str()).copied()),
                    Operand::Copy(place) | Operand::Move(place) => {
                        let ty = place_type(body, types, place);
                        runs.extend(targets.of_call(at, body, operand, ty, types));
                    }
                    _ => {}
                }
            }
            // `[THR-20]`, `[RC-4]` — what the thread is given ends there: the drops its value may run (a
            // lambda's captured state, a count of a `thread_shared` object, which may be the last); and its
            // result, which ends there when no one joins it (`JoinHandle[T]`'s `T`).
            if let Terminator::Call { dest, .. } = &block.terminator
                && let TyKind::Struct(id) = types.kind(place_type(body, types, dest))
                && let Some((_, results)) = &types.struct_def(*id).origin
            {
                for &result in results {
                    runs.extend(drop_runs.of(result, targets).bodies.iter().map(|&(drop, _)| drop));
                }
            }
            for operand in args {
                if let Operand::Copy(place) | Operand::Move(place) = operand {
                    runs.extend(drop_runs.of(place_type(body, types, place), targets).bodies.iter().map(|&(drop, _)| drop));
                }
            }
            runs.sort_unstable();
            runs.dedup();
            // The first function that does it, and each module variable it reaches: changed, or read.
            let mut who: Option<usize> = None;
            let mut reached: Vec<(String, bool, ember_types::Ty)> = Vec::new();
            for &task in &runs {
                let summary = &summaries[task];
                let changes = summary.changes.iter().chain(&summary.reshapes).chain(&summary.ends).map(|fact| (fact, true));
                for (fact, changed) in changes.chain(summary.reads.iter().map(|fact| (fact, false))) {
                    let Root::Static(name, ty) = &fact.reach.root else { continue };
                    if !types.is_module_variable(ember_span::Symbol::intern(name)) {
                        continue;
                    }
                    who.get_or_insert_with(|| program_chain(fact, |at| in_std(&bodies[at])).0.last().copied().unwrap_or(task));
                    let shown = name.rsplit('.').next().unwrap_or(name).to_string();
                    match reached.iter_mut().find(|(seen, _, _)| *seen == shown) {
                        Some((_, was, _)) => *was |= changed,
                        None => reached.push((shown, changed, *ty)),
                    }
                }
            }
            let Some(who) = who else { continue };
            reached.sort_by(|a, b| a.0.cmp(&b.0));
            let quoted: Vec<String> = reached.iter().map(|(name, _, _)| format!("`{name}`")).collect();
            let names = match quoted.as_slice() {
                [one] => one.clone(),
                [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
                [] => continue,
            };
            let verb = if reached.iter().any(|(_, changed, _)| *changed) { "changes" } else { "reads" };
            let (them, kind) = match reached.len() {
                1 => ("it", "a module variable"),
                2 => ("both", "module variables"),
                _ => ("them all", "module variables"),
            };
            let (subject, its_body, lock) = named_with_lock(&bodies[who], types, &reached[0].0);
            let declared = reached
                .iter()
                .map(|(name, _, ty)| ember_diag::guarded_static_mark(name, &lock, &types.written(*ty)))
                .collect::<Vec<_>>()
                .join(" and ");
            let span = block.call_span.unwrap_or(block.terminator_span);
            sink.emit(
                ember_diag::Diagnostic::error(
                    ember_diag::codes::E7012,
                    span,
                    format!("{subject} {verb} {names}, {kind} of the main thread, and this {runner} runs beside it; guard {them} with one lock: write `from std.sync import Lock` at the top, `static {lock}: Lock = Lock()`, then {declared}, and put {its_body} inside `with {lock}:`"),
                )
                .note("a `static mut` is reached only by the main thread and the tasks and jobs of the scopes it opens, unless it is `guarded by` a lock [THR-20], [STA-1]"),
            );
        }
    }
}

/// The first module variable a summary reaches (changed, reshaped, ended or read), by its name, and the line.
fn module_variable_reached(summary: &Summary, types: &TypeTable) -> Option<(String, Span)> {
    summary.changes.iter().chain(&summary.reshapes).chain(&summary.ends).chain(&summary.reads).find_map(|fact| match &fact.reach.root {
        Root::Static(name, _) if types.is_module_variable(ember_span::Symbol::intern(name)) => Some((name.rsplit('.').next().unwrap_or(name).to_string(), fact.at)),
        _ => None,
    })
}

/// `[THR-11]`, `[THR-20]`, `[EFF-1]` (review 4) — what is not followed yet is refused, not trusted: a generator that
/// reaches a module variable (the summaries do not yet count a generator's `next` as running its body), and a
/// scoped task or job that does (`[THR-11]`'s check of a module variable a scope's tasks reach is M9's).
pub(crate) fn check_module_variables_not_followed(bodies: &[Body], types: &TypeTable, summaries: &[Summary], targets: &Targets, sink: &mut ember_diag::Sink) {
    let index: HashMap<&str, usize> = bodies.iter().enumerate().map(|(at, body)| (body.symbol.as_str(), at)).collect();
    for (at, body) in bodies.iter().enumerate() {
        if in_std(body) {
            continue;
        }
        if crate::coroutines::is_generator(body, types)
            && let Some((name, line)) = module_variable_reached(&summaries[at], types)
        {
            sink.emit(
                ember_diag::Diagnostic::error(ember_diag::codes::E0900, line, format!("a generator that reaches the module variable `{name}` is not built by this compiler yet"))
                    .note("each `next` runs the generator's body ([EFF-1]), which the checks of a module variable do not follow yet; read or change it outside the generator"),
            );
        }
        for block in &body.blocks {
            let Terminator::Call { func: func @ FuncRef::Builtin { which: Builtin::ScopeSpawn { .. } | Builtin::JobScopeSubmit { .. }, .. }, args, .. } = &block.terminator else { continue };
            let reached = builtin_runs(at, body, func, args, types, &index, targets).into_iter().find_map(|run| module_variable_reached(&summaries[run], types));
            if let Some((name, _)) = reached {
                sink.emit(
                    ember_diag::Diagnostic::error(
                        ember_diag::codes::E0900,
                        block.call_span.unwrap_or(block.terminator_span),
                        format!("a task of a scope that reaches the module variable `{name}` is not built by this compiler yet"),
                    )
                    .note("a module variable the tasks of a scope reach is checked as any other place ([THR-20], [THR-11]), which is not built for module variables yet"),
                );
            }
        }
    }
}

/// `[LT-3]` (review 4) — a function whose result may hold a view into a `static mut` (`fn head() -> StrView:
/// return LOG[0]`) is `E0900`: the summary does not yet record what a result points into, so a caller would
/// keep no loan of the module variable (LT-3's rule, M5's part).
pub(crate) fn check_views_of_module_variables_returned(bodies: &[Body], types: &TypeTable, sink: &mut ember_diag::Sink) {
    for body in bodies {
        let result = body.local(ember_mir::RETURN_LOCAL).ty;
        if !types.may_be_view(result) || in_std(body) {
            continue;
        }
        let pointers = pointers_of(body, types);
        let mut out = Vec::new();
        let steps = if points(types, result) { vec![Step::Deref] } else { vec![Step::Field(0), Step::Deref] };
        reaches(body, &pointers, &Spot { local: ember_mir::RETURN_LOCAL, steps }, 0, &mut out);
        let Some(name) = out.iter().find_map(|reach| match &reach.root {
            Root::Static(name, _) if types.is_module_variable(ember_span::Symbol::intern(name)) => Some(name.rsplit('.').next().unwrap_or(name).to_string()),
            _ => None,
        }) else { continue };
        let span = body.blocks.iter().flat_map(|block| &block.stmts).find_map(|stmt| match &stmt.kind {
            StmtKind::Assign { place, .. } if place.local == ember_mir::RETURN_LOCAL => Some(stmt.span),
            _ => None,
        });
        sink.emit(
            ember_diag::Diagnostic::error(
                ember_diag::codes::E0900,
                span.or_else(|| body.blocks.first().map(|block| block.terminator_span)).unwrap_or(Span::DUMMY),
                format!("a function returning a view of the module variable `{name}` is not built by this compiler yet"),
            )
            .note("what a function's result points into, a `static mut` among them, is recorded with it by [LT-3], which is not built; return a copy of what it views"),
        );
    }
}

/// `[DIA-27]` — the part of a fact's chain a message names: the program's own functions, up to its first call
/// into the standard library (code the program did not write), and the line that does it, that call's line
/// in the program or the fact's own.
pub(crate) fn program_chain(fact: &Fact, in_std: impl Fn(usize) -> bool) -> (Vec<usize>, Span) {
    let mut chain = Vec::new();
    for &(callee, call) in &fact.chain {
        if in_std(callee) {
            return (chain, call);
        }
        chain.push(callee);
    }
    (chain, fact.at)
}

/// Whether `body` is the standard library's.
pub(crate) fn in_std(body: &Body) -> bool {
    ember_branding::symbol_path_starts_with(&body.symbol, &[ember_branding::STD_PACKAGE])
}

/// `[THR-20]` — how `E7012` names the function that reaches a module variable, its body, and the lock it
/// offers: a method by its type (`Ticket.drop`, `TICKET_LOCK`), a function by its name (`tick`,
/// `TICK_LOCK`), a lambda by the module variable it reaches first (`FRAME_LOCK`).
fn named_with_lock(body: &Body, types: &TypeTable, first: &str) -> (String, String, String) {
    if body.is_lambda {
        return ("the lambda this thread runs".to_string(), "the lambda's body".to_string(), format!("{}_LOCK", first.to_uppercase()));
    }
    let mut names = body.name.rsplit('.');
    let name = names.next().unwrap_or(&body.name).to_string();
    let owner = match body.class_owner {
        Some(class) => Some(types.class_def(class).name.to_string()),
        None if is_method(body) => names.next().map(str::to_string),
        None => None,
    };
    let (shown, lock) = match owner {
        Some(owner) => (format!("{owner}.{name}"), format!("{}_LOCK", owner.to_uppercase())),
        None => (name.clone(), format!("{}_LOCK", name.to_uppercase())),
    };
    (format!("`{shown}`"), format!("`{shown}`'s body"), lock)
}

/// `[DRP-8]` — a built-in that ends elements of the list its first argument points at (a clear, a
/// truncation, an element ended in place): the place the list is, and its type.
fn ended_elements(body: &Body, types: &TypeTable, which: &Builtin, args: &[Operand]) -> Option<(Place, Ty)> {
    if !matches!(which, Builtin::ArrayClear | Builtin::ArrayTruncate | Builtin::SoA { op: SoAOp::Clear | SoAOp::Truncate | SoAOp::DropAt, .. }) {
        return None;
    }
    let Some(Operand::Copy(place) | Operand::Move(place)) = args.first() else { return None };
    let ty = place_type(body, types, place);
    match *types.kind(ty) {
        TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => {
            let mut through = place.clone();
            through.projection.push(Projection::Deref);
            Some((through, inner))
        }
        TyKind::Vec { .. } => Some((place.clone(), ty)),
        _ => None,
    }
}

/// `[DRP-8]` — ending the value in `place`, of type `ty`, at `span` makes the changes its `drop`s make
/// outside its object: each fact of a `drop` it may run that is rooted at a static, or that leaves the
/// object through something the object points at (a link, a view, a handle), as a call of that `drop`.
/// What a `drop` does to its own object ends with the object.
#[allow(clippy::too_many_arguments)]
fn ended_facts(
    body: &Body,
    types: &TypeTable,
    bodies: &[Body],
    targets: &Targets,
    pointers: &HashMap<LocalId, HashSet<Target>>,
    summaries: &[Summary],
    drop_runs: &DropRuns,
    place: &Place,
    ty: Ty,
    span: Span,
) -> Vec<(What, Fact)> {
    let mut found = Vec::new();
    for &(drop, own) in &drop_runs.of(ty, targets).bodies {
        let summary = &summaries[drop];
        // The `drop`'s object: through its `this` view (a struct's), or its handle (a class's).
        let viewed = matches!(bodies[drop].locals.get(1).map(|local| types.kind(local.ty)), Some(TyKind::Ref { .. } | TyKind::Ptr { .. }));
        for (what, facts) in [(What::Change, &summary.changes), (What::Read, &summary.reads), (What::Reshape, &summary.reshapes), (What::Add, &summary.adds), (What::Remove, &summary.removes), (What::End, &summary.ends)] {
            for fact in facts {
                let mut chain = vec![(drop, span)];
                chain.extend_from_slice(&fact.chain);
                let rest = match &fact.reach.root {
                    Root::Static(..) => {
                        found.push((what, Fact { reach: fact.reach.clone(), chain, at: fact.at, op: fact.op }));
                        continue;
                    }
                    // Past its own object only through something the object points at.
                    Root::Param(0) => match fact.reach.path.split_first() {
                        Some((Step::Deref, rest)) if rest.contains(&Step::Deref) => rest,
                        _ => continue,
                    },
                    Root::Param(_) => continue,
                };
                let mut steps = steps_of(body, types, place);
                if own {
                    if !viewed {
                        steps.push(Step::Deref);
                    }
                    steps.extend_from_slice(rest);
                } else {
                    // A part's `drop`: somewhere the value points at.
                    steps.push(Step::Deref);
                }
                let mut out = Vec::new();
                reaches(body, pointers, &Spot { local: place.local, steps }, 0, &mut out);
                found.extend(out.into_iter().map(|reach| (what, Fact { reach, chain: chain.clone(), at: fact.at, op: fact.op })));
            }
        }
    }
    found
}

/// The bodies `body`'s calls may run, as `body_facts` resolves them: direct calls, calls through a
/// function value, `virtual` and `dyn` methods, and the functions a built-in runs itself.
fn callees_of(at: usize, body: &Body, bodies: &[Body], types: &TypeTable, index: &HashMap<&str, usize>, targets: &Targets, drop_runs: &DropRuns) -> Vec<usize> {
    let mut out = Vec::new();
    for block in &body.blocks {
        // `[DRP-8]` — an end runs each `drop` it may reach: a call of it.
        for stmt in &block.stmts {
            if let StmtKind::Drop { place, .. } = &stmt.kind {
                out.extend(drop_runs.of(place_type(body, types, place), targets).bodies.iter().map(|&(drop, _)| drop));
            }
        }
        let Terminator::Call { func, args, .. } = &block.terminator else { continue };
        if let FuncRef::Builtin { which, .. } = func
            && let Some((_, list)) = ended_elements(body, types, which, args)
        {
            out.extend(drop_runs.of(list, targets).bodies.iter().map(|&(drop, _)| drop));
        }
        out.extend(call_targets(at, body, func, args, bodies, types, index, targets));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// `[EFF-24]`, `[DRP-8]` (M6a) — what a checked body's calls and ends may run, by symbol: each call's
/// bodies by its block (for a call through a value, a `dyn` or a virtual method, each one it may be; for a
/// built-in, each function it is given), and each end's `drop`s by its point (a value ended, or the
/// elements a built-in ends).
#[derive(Default)]
pub(crate) struct Runs {
    /// Each call's bodies, by symbol and the type that declares them.
    pub(crate) calls: HashMap<usize, Vec<(String, String)>>,
    /// Each end's `drop`s, by symbol and the type whose `drop` each is.
    pub(crate) ends: HashMap<(usize, usize), Vec<(String, String)>>,
    /// Each module variable the body names, by its name: the one root of its places
    /// (`borrows::module_variable_roots`).
    pub(crate) statics: HashMap<String, LocalId>,
}

/// The [`Runs`] of `body`, body `at` of the program (a form of it, when its `dyn` calls are made direct).
#[allow(clippy::too_many_arguments)]
pub(crate) fn runs_of(at: usize, body: &Body, bodies: &[Body], types: &TypeTable, index: &HashMap<&str, usize>, targets: &Targets, drop_runs: &DropRuns) -> Runs {
    let symbols = |found: Vec<usize>| -> Vec<(String, String)> { found.into_iter().map(|at| (bodies[at].symbol.clone(), owner_of(&bodies[at], types).unwrap_or_default())).collect() };
    let mut runs = Runs::default();
    let roots = crate::borrows::module_variable_roots(body);
    for block in &body.blocks {
        if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::LazyStatic { name, .. } | Builtin::ConstStatic { name, .. } | Builtin::ForeignStatic { symbol: name, .. }, .. }, dest, .. } = &block.terminator
            && dest.projection.is_empty()
        {
            runs.statics.entry(name.as_str().to_string()).or_insert_with(|| roots.get(&dest.local).copied().unwrap_or(dest.local));
        }
    }
    for (at_block, block) in body.blocks.iter().enumerate() {
        for (at_stmt, stmt) in block.stmts.iter().enumerate() {
            if let StmtKind::Drop { place, .. } = &stmt.kind {
                let drops = symbols(drop_runs.of(place_type(body, types, place), targets).bodies.iter().map(|&(drop, _)| drop).collect());
                if !drops.is_empty() {
                    runs.ends.insert((at_block, at_stmt), drops);
                }
            }
        }
        let Terminator::Call { func, args, .. } = &block.terminator else { continue };
        let called = symbols(call_targets(at, body, func, args, bodies, types, index, targets));
        if !called.is_empty() {
            runs.calls.insert(at_block, called);
        }
        if let FuncRef::Builtin { which, .. } = func
            && let Some((_, list)) = ended_elements(body, types, which, args)
        {
            let drops = symbols(drop_runs.of(list, targets).bodies.iter().map(|&(drop, _)| drop).collect());
            if !drops.is_empty() {
                runs.ends.insert((at_block, block.stmts.len()), drops);
            }
        }
    }
    runs
}

/// The type a method belongs to, by its written name: a class's, or the type a method with `this` is
/// declared in (`Ticket` for `Ticket.drop`).
pub(crate) fn owner_of(body: &Body, types: &TypeTable) -> Option<String> {
    match body.class_owner {
        Some(class) => Some(types.class_def(class).name.to_string()),
        None if is_method(body) => body.name.rsplit('.').nth(1).map(str::to_string),
        None => None,
    }
}

/// `[DRP-8]`, `[DIA-27]` — how a message names what a built-in does to the list or text it changes.
fn builtin_op(which: &Builtin) -> Option<&'static str> {
    Some(match which {
        Builtin::ArrayPush | Builtin::StringPush | Builtin::StringPushChar => "pushes to",
        Builtin::ArrayExtend => "extends",
        Builtin::ArrayInsert | Builtin::StringInsert => "inserts into",
        Builtin::ArrayPop { .. } => "pops from",
        Builtin::ArrayRemove | Builtin::ArraySwapRemove | Builtin::StringRemove => "removes from",
        Builtin::ArrayClear => "clears",
        Builtin::ArrayTruncate | Builtin::StringTruncate => "truncates",
        Builtin::ArraySort => "sorts",
        Builtin::ArrayReverse => "reverses",
        _ => return None,
    })
}

/// `[EFF-1]`, `[STA-3]`, `[DRP-8]` (review 4) — the bodies a built-in call runs: each function it is given
/// (a named one, or a function value's targets), a static's first-access initialiser, the `clone` a copy of a
/// value runs (its type's, or its parts' where it has none), and the written `Debug` a format runs.
#[allow(clippy::too_many_arguments)]
fn builtin_runs(at: usize, body: &Body, func: &FuncRef, args: &[Operand], types: &TypeTable, index: &HashMap<&str, usize>, targets: &Targets) -> Vec<usize> {
    let FuncRef::Builtin { which, arg_ty } = func else { return Vec::new() };
    let mut out = Vec::new();
    for operand in args {
        match operand {
            Operand::Const(Const::Fn(symbol)) => out.extend(index.get(symbol.as_str()).copied()),
            // A function value a built-in runs (a thread, a task, a job, a parallel loop's body); one it only
            // stores (`hs.push(owns log_it)`) runs nothing here.
            Operand::Copy(place) | Operand::Move(place)
                if matches!(which, Builtin::ThreadSpawn | Builtin::ScopeSpawn { .. } | Builtin::JobSubmit { .. } | Builtin::JobScopeSubmit { .. } | Builtin::ParallelFor { .. }) =>
            {
                let ty = place_type(body, types, place);
                if matches!(types.kind(ty), TyKind::Fn { .. }) {
                    out.extend(targets.of_call(at, body, operand, ty, types));
                }
            }
            _ => {}
        }
    }
    match which {
        Builtin::LazyStatic { init, .. } => out.extend(index.get(init.as_str()).copied()),
        Builtin::Format | Builtin::FormatWith(_) | Builtin::Print | Builtin::EPrint | Builtin::PrintPiece { .. } => {
            out.extend(types.debug_writers_reached(*arg_ty).into_iter().filter_map(|writer| index.get(writer.as_ref() as &str).copied()));
        }
        Builtin::ArrayClone { elem } | Builtin::CloneParts { ty: elem } => clone_runs(*elem, types, index, &mut HashSet::new(), &mut out),
        _ => {}
    }
    out
}

/// The `clone` bodies a copy of a value of `ty` runs: its type's own, or, where it has none, its parts'.
fn clone_runs(ty: Ty, types: &TypeTable, index: &HashMap<&str, usize>, seen: &mut HashSet<Ty>, out: &mut Vec<usize>) {
    if !seen.insert(ty) {
        return;
    }
    if matches!(types.kind(ty), TyKind::Struct(_) | TyKind::Enum(_))
        && let Some(&clone) = index.get(types.method_symbol(ty, None, "clone", None).as_str())
    {
        out.push(clone);
        return;
    }
    match types.kind(ty).clone() {
        TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => clone_runs(elem, types, index, seen, out),
        TyKind::Tuple(items) => items.into_iter().for_each(|item| clone_runs(item, types, index, seen, out)),
        TyKind::Enum(id) => {
            for field in types.enum_def(id).variants.iter().flat_map(|variant| &variant.fields) {
                clone_runs(field.ty, types, index, seen, out);
            }
        }
        TyKind::Struct(id) => {
            let def = types.struct_def(id);
            for field in &def.fields {
                clone_runs(field.ty, types, index, seen, out);
            }
            if let Some((_, args)) = &def.origin {
                for &arg in args {
                    clone_runs(arg, types, index, seen, out);
                }
            }
        }
        _ => {}
    }
}

/// The bodies one call may run, as `body_facts` resolves them.
#[allow(clippy::too_many_arguments)]
fn call_targets(at: usize, body: &Body, func: &FuncRef, args: &[Operand], bodies: &[Body], types: &TypeTable, index: &HashMap<&str, usize>, targets: &Targets) -> Vec<usize> {
    let mut out = Vec::new();
    {
        match func {
            // A call a form of its body made direct stands for the bodies it was made from.
            FuncRef::Direct { symbol, .. } => match index.get(symbol.as_str()) {
                Some(&callee) => out.push(callee),
                None => out.extend(crate::dyn_kinds::formed_symbols(symbol).into_iter().flatten().filter_map(|symbol| index.get(symbol).copied())),
            },
            FuncRef::Indirect { operand, ty, .. } => out.extend(targets.of_call(at, body, operand, *ty, types)),
            FuncRef::Virtual { owner, slot, .. } => out.extend(
                bodies
                    .iter()
                    .enumerate()
                    .filter(|(_, candidate)| candidate.class_virtual_slot == Some(*slot) && candidate.class_owner.is_some_and(|class| below(types, class, *owner)))
                    .map(|(callee, _)| callee),
            ),
            FuncRef::Interface { interfaces, slot, .. } => out.extend(targets.of_dyn(interfaces, *slot).iter().copied()),
            FuncRef::Builtin { .. } => out.extend(builtin_runs(at, body, func, args, types, index, targets)),
            _ => {}
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// The steps of a place. MIR reaches a class object's field straight through its handle (`n.next`
/// of a `n: Node`): that field is through the handle, a `Deref` the steps write.
fn steps_of(body: &Body, types: &TypeTable, place: &Place) -> Vec<Step> {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    let mut steps = Vec::with_capacity(place.projection.len());
    for projection in &place.projection {
        if matches!(projection, Projection::Field(_)) && matches!(types.kind(ty), TyKind::Class(_)) {
            steps.push(Step::Deref);
        }
        steps.push(match projection {
            Projection::Field(field) => Step::Field(*field),
            Projection::Index(_) | Projection::ConstIndex(_) => Step::Elements,
            Projection::Downcast(variant) => Step::Variant(*variant),
            Projection::Column(column) => Step::Column(*column),
            Projection::Deref => Step::Deref,
        });
        ty = project_type(types, ty, projection, &mut variant);
    }
    steps
}

/// §VII.1 — whether writing a value of this type is a reshape: it owns memory the write may free or
/// move (a list, a string, a box, an object's handle, a `dyn` value, or anything holding one), or it
/// is an enum whose variant the write may change. A number, a `bool`, a view or a pointer is not.
pub(crate) fn reshaped_by_write(types: &TypeTable, ty: ember_types::Ty, depth: usize) -> bool {
    if depth > 8 {
        return true;
    }
    match types.kind(ty) {
        TyKind::Vec { .. } | TyKind::Class(_) | TyKind::Dyn { .. } | TyKind::ClassInterface(_) => true,
        TyKind::Struct(id) => types.struct_def(*id).fields.iter().any(|field| reshaped_by_write(types, field.ty, depth + 1)),
        TyKind::Enum(id) => types.enum_def(*id).variants.iter().any(|variant| !variant.fields.is_empty()),
        TyKind::Tuple(items) => items.iter().any(|item| reshaped_by_write(types, *item, depth + 1)),
        TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => reshaped_by_write(types, *elem, depth + 1),
        _ => false,
    }
}

/// §VII.1 — a built-in that adds, removes, reorders or replaces the elements of the list, string or
/// table its first argument points at, or replaces the value it points at.
pub(crate) fn reshaping_builtin(types: &TypeTable, which: &Builtin) -> bool {
    match which {
        Builtin::ArrayPush
        | Builtin::ArrayInsert
        | Builtin::ArrayExtend
        | Builtin::ArrayPop { .. }
        | Builtin::ArrayRemove
        | Builtin::ArraySwapRemove
        | Builtin::ArrayDrain
        | Builtin::ArrayClear
        | Builtin::ArrayTruncate
        | Builtin::ArrayReserve
        | Builtin::ArrayReserveHint { .. }
        | Builtin::ArraySort
        | Builtin::ArrayReverse
        | Builtin::ArraySwap
        | Builtin::StringPush
        | Builtin::StringPushChar
        | Builtin::StringInsert
        | Builtin::StringRemove
        | Builtin::StringTruncate
        | Builtin::ArenaReset => true,
        Builtin::MemReplace { elem } | Builtin::MemTake { elem, .. } | Builtin::MemSwap { elem } => reshaped_by_write(types, *elem, 0),
        Builtin::SoA { op, .. } => matches!(
            op,
            SoAOp::Push | SoAOp::TryPush | SoAOp::TakeLast | SoAOp::Clear | SoAOp::Reserve | SoAOp::SwapRemove | SoAOp::Swap | SoAOp::Truncate | SoAOp::MoveSlot | SoAOp::DropAt | SoAOp::SetLen
        ),
        _ => false,
    }
}

/// `[EFF-24]` — a built-in that only adds to the end of the list or string its first argument points
/// at, or makes room there.
fn adding_builtin(which: &Builtin) -> bool {
    matches!(
        which,
        Builtin::ArrayPush
            | Builtin::ArrayExtend
            | Builtin::ArrayReserve
            | Builtin::ArrayReserveHint { .. }
            | Builtin::StringPush
            | Builtin::StringPushChar
            | Builtin::SoA { op: SoAOp::Push | SoAOp::TryPush | SoAOp::Reserve, .. }
    )
}

/// Whether a value of this type points at memory the function may reach through it: a `StrView` and a
/// C string at their text, too.
fn points(types: &TypeTable, ty: ember_types::Ty) -> bool {
    matches!(
        types.kind(ty),
        TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Class(_) | TyKind::Span { .. } | TyKind::Vec { .. } | TyKind::Dyn { .. } | TyKind::ClassInterface(_) | TyKind::Str | TyKind::CStr
    )
}

/// A built-in whose result is memory it has just made, which nothing the function had reaches.
fn fresh_builtin(which: &Builtin) -> bool {
    matches!(which, Builtin::ArrayNew | Builtin::StringNew | Builtin::Input)
}

/// What each local holding a pointer may point at, over the whole body (flow-insensitive: every
/// value it is ever given).
fn pointers_of(body: &Body, types: &TypeTable) -> HashMap<LocalId, HashSet<Target>> {
    let mut map: HashMap<LocalId, HashSet<Target>> = HashMap::new();
    let pointer = |local: LocalId| (local.0 as usize) < body.locals.len() && points(types, body.locals[local.0 as usize].ty);
    for block in &body.blocks {
        for stmt in &block.stmts {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            // `[EFF-24]` (review 4) — a local holding views inside it (a struct, a tuple, an `Option`, a
            // closure's state, made whole or a field at a time) points at what each of them points at,
            // through any of its parts.
            if !pointer(place.local) && (place.local.0 as usize) < body.locals.len() {
                let held: Vec<&Operand> = match rvalue {
                    Rvalue::Aggregate { operands, .. } if place.projection.is_empty() => operands.iter().collect(),
                    Rvalue::Use(operand) | Rvalue::Cast { operand, .. } if !place.projection.is_empty() => vec![operand],
                    _ => Vec::new(),
                };
                for operand in held {
                    if let Operand::Copy(from) | Operand::Move(from) = operand
                        && points(types, place_type(body, types, from))
                    {
                        let mut steps = steps_of(body, types, from);
                        steps.push(Step::Deref);
                        map.entry(place.local).or_default().insert(Target::At(Spot { local: from.local, steps }));
                    }
                }
                if let Rvalue::Ref { place: lent, .. } = rvalue
                    && !place.projection.is_empty()
                {
                    map.entry(place.local).or_default().insert(Target::At(Spot { local: lent.local, steps: steps_of(body, types, lent) }));
                }
            }
            if !place.projection.is_empty() || !pointer(place.local) {
                continue;
            }
            let into = map.entry(place.local).or_default();
            match rvalue {
                Rvalue::Ref { place: lent, .. } => {
                    into.insert(Target::At(Spot { local: lent.local, steps: steps_of(body, types, lent) }));
                }
                Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) | Rvalue::Cast { operand: Operand::Copy(from) | Operand::Move(from), .. } => {
                    let mut steps = steps_of(body, types, from);
                    steps.push(Step::Deref);
                    into.insert(Target::At(Spot { local: from.local, steps }));
                }
                Rvalue::Use(Operand::Const(_)) | Rvalue::Cast { operand: Operand::Const(_), .. } => {}
                _ => {
                    into.insert(Target::Anything);
                }
            }
        }
        // A pointer a call returns points at what its pointer arguments do, or at anything; one to a
        // `static` at the static.
        if let Terminator::Call { func, args, dest, .. } = &block.terminator
            && dest.projection.is_empty()
            && pointer(dest.local)
        {
            let into = map.entry(dest.local).or_default();
            if let FuncRef::Builtin { which: Builtin::LazyStatic { name, ty, .. } | Builtin::ConstStatic { name, ty } | Builtin::ForeignStatic { symbol: name, ty, .. }, .. } = func {
                into.insert(Target::Static(name.to_string(), *ty));
                continue;
            }
            if let FuncRef::Builtin { which, .. } = func
                && fresh_builtin(which)
            {
                continue;
            }
            let mut any = false;
            for operand in args {
                if let Operand::Copy(from) | Operand::Move(from) = operand
                    && points(types, place_type(body, types, from))
                {
                    let mut steps = steps_of(body, types, from);
                    steps.push(Step::Deref);
                    into.insert(Target::At(Spot { local: from.local, steps }));
                    any = true;
                }
            }
            if !any {
                into.insert(Target::Anything);
            }
        }
    }
    map
}

/// The parameter places a spot of the body reaches: a parameter's through a pointer, or through a
/// local pointing at one.
fn reaches(body: &Body, pointers: &HashMap<LocalId, HashSet<Target>>, spot: &Spot, depth: usize, out: &mut Vec<Reach>) {
    let index = spot.local.0 as usize;
    if (1..=body.arg_count).contains(&index) {
        if spot.steps.contains(&Step::Deref) {
            let mut path = spot.steps.clone();
            path.truncate(MAX_PATH);
            out.push(Reach { root: Root::Param(index - 1), path });
        }
        return;
    }
    // `[EFF-24]` (review 4) — through a view kept in a part of a local (`t.0`, `h.items`): what the local's
    // views point at, past the first `Deref`; a place reached through one nothing records reaches anything.
    if !matches!(spot.steps.first(), Some(Step::Deref)) {
        let Some(at) = spot.steps.iter().position(|step| *step == Step::Deref) else { return };
        match pointers.get(&spot.local) {
            Some(held) if depth <= 16 => {
                for target in held {
                    match target {
                        Target::At(inner) => {
                            let mut steps = inner.steps.clone();
                            steps.extend_from_slice(&spot.steps[at + 1..]);
                            reaches(body, pointers, &Spot { local: inner.local, steps }, depth + 1, out);
                        }
                        Target::Static(name, ty) => {
                            let mut path = spot.steps[at + 1..].to_vec();
                            path.truncate(MAX_PATH);
                            out.push(Reach { root: Root::Static(name.clone(), *ty), path });
                        }
                        Target::Anything => anything(body, out),
                    }
                }
            }
            _ => anything(body, out),
        }
        return;
    }
    // Through a local's pointer: `(*p).rest` is the place `p` points at, then `rest`.
    let Some(Step::Deref) = spot.steps.first() else { return };
    if depth > 16 {
        anything(body, out);
        return;
    }
    let Some(held) = pointers.get(&spot.local) else {
        anything(body, out);
        return;
    };
    for target in held {
        match target {
            Target::At(at) => {
                let mut steps = at.steps.clone();
                steps.extend_from_slice(&spot.steps[1..]);
                reaches(body, pointers, &Spot { local: at.local, steps }, depth + 1, out);
            }
            // The static's own place, then the steps past the pointer.
            Target::Static(name, ty) => {
                let mut path = spot.steps[1..].to_vec();
                path.truncate(MAX_PATH);
                out.push(Reach { root: Root::Static(name.clone(), *ty), path });
            }
            Target::Anything => anything(body, out),
        }
    }
}

/// Anything any parameter reaches.
fn anything(body: &Body, out: &mut Vec<Reach>) {
    for param in 0..body.arg_count {
        out.push(Reach { root: Root::Param(param), path: vec![Step::Deref] });
    }
}

#[allow(clippy::too_many_arguments)]
fn body_facts(
    at: usize,
    body: &Body,
    bodies: &[Body],
    types: &TypeTable,
    index: &HashMap<&str, usize>,
    targets: &Targets,
    pointers: &HashMap<LocalId, HashSet<Target>>,
    regions: Option<&Regions>,
    summaries: &[Summary],
    drop_runs: &DropRuns,
) -> Vec<(What, Fact)> {
    let mut found = Vec::new();
    // The line being read: each of its own facts is made there.
    let line = std::cell::Cell::new(Span::DUMMY);
    let op: std::cell::Cell<Option<&'static str>> = std::cell::Cell::new(None);
    let mut own = |what: What, place: &Place, found: &mut Vec<(What, Fact)>| {
        let mut out = Vec::new();
        reaches(body, pointers, &Spot { local: place.local, steps: steps_of(body, types, place) }, 0, &mut out);
        found.extend(out.into_iter().map(|reach| (what, Fact { reach, chain: Vec::new(), at: line.get(), op: op.get() })));
    };
    let read = |operand: &Operand, own: &mut dyn FnMut(What, &Place, &mut Vec<(What, Fact)>), found: &mut Vec<(What, Fact)>| {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            own(What::Read, place, found);
        }
    };
    // `[BRW-21]` — the view parameters (those whose value carries a region), and whether one is still
    // used at a point: its regions are live there.
    // `[CLS-2]` — the locals holding `this` as a whole, and an operand that is one of them.
    let aliases = this_aliases(body);
    let whole = |operand: &Operand| matches!(operand, Operand::Copy(place) | Operand::Move(place) if whole_place(&aliases, place));
    let let_out = Fact { reach: Reach { root: Root::Param(0), path: Vec::new() }, chain: Vec::new(), at: Span::DUMMY, op: None };
    let views: Vec<usize> = match regions {
        Some(regions) => (0..body.arg_count).filter(|&param| !regions.local_regions(LocalId(param as u32 + 1)).is_empty()).collect(),
        None => Vec::new(),
    };
    let live = |param: usize, point: Point| {
        regions.is_some_and(|regions| regions.local_regions(LocalId(param as u32 + 1)).iter().any(|slot| regions.contains(slot.region, point)))
    };
    // `[BRW-1]` — of them, the `ref mut` ones: a slot holding a changeable view.
    let unique: Vec<usize> = match regions {
        Some(regions) => views.iter().copied().filter(|&param| regions.local_regions(LocalId(param as u32 + 1)).iter().any(|slot| slot.unique)).collect(),
        None => Vec::new(),
    };
    // Each fact found since `start` on a line at `at`, which `after` follows: a reshape or an end, one
    // more fact for each view parameter still used after it; a read or a change, one for each `ref mut`
    // parameter still used at it; neither when it was made through the parameter itself.
    let per_parameter = |start: usize, at: Point, after: Point, found: &mut Vec<(What, Fact)>| {
        let mut more = Vec::new();
        for (what, fact) in &found[start..] {
            let (params, point, kind): (&[usize], Point, fn(usize) -> What) = match what {
                What::Reshape | What::End => (&views, after, What::UsedAfter),
                What::Read => (&unique, at, What::ReadWhile),
                What::Change => (&unique, at, What::ChangedWhile),
                _ => continue,
            };
            for &param in params {
                if live(param, point) && !through_itself(param, &fact.reach) {
                    more.push((kind(param), fact.clone()));
                }
            }
        }
        found.extend(more);
    };
    for (at_block, block) in body.blocks.iter().enumerate() {
        for (at_stmt, stmt) in block.stmts.iter().enumerate() {
            let start = found.len();
            line.set(stmt.span);
            op.set(None);
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    own(What::Change, place, &mut found);
                    if reshaped_by_write(types, place_type(body, types, place), 0) {
                        own(What::Reshape, place, &mut found);
                        own(What::Remove, place, &mut found);
                    }
                    match rvalue {
                        Rvalue::Use(operand) | Rvalue::Cast { operand, .. } | Rvalue::UnaryOp { operand, .. } | Rvalue::Repeat { value: operand, .. } => {
                            read(operand, &mut own, &mut found);
                        }
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            read(lhs, &mut own, &mut found);
                            read(rhs, &mut own, &mut found);
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            for operand in operands {
                                read(operand, &mut own, &mut found);
                            }
                        }
                        Rvalue::Discriminant(place) => own(What::Read, place, &mut found),
                        // What is lent is changed or read by whoever uses the view: followed there.
                        Rvalue::Ref { .. } => {}
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    own(What::Change, dest, &mut found);
                    own(What::Change, overflow, &mut found);
                    read(lhs, &mut own, &mut found);
                    read(rhs, &mut own, &mut found);
                }
                StmtKind::Drop { place, .. } => {
                    own(What::End, place, &mut found);
                    let ty = place_type(body, types, place);
                    found.extend(ended_facts(body, types, bodies, targets, pointers, summaries, drop_runs, place, ty, stmt.span));
                }
                _ => {}
            }
            // `[CLS-2]` — `this` stored (into a field, an element, a struct or a closure's state) or
            // returned; a copy or view of it in another local is not yet a use.
            if let StmtKind::Assign { place, rvalue } = &stmt.kind {
                let uses = match rvalue {
                    Rvalue::Use(operand) | Rvalue::Cast { operand, .. } => whole(operand),
                    Rvalue::Ref { place: lent, .. } => whole_place(&aliases, lent),
                    Rvalue::Aggregate { operands, .. } => operands.iter().any(&whole),
                    _ => false,
                };
                let copied = place.projection.is_empty() && place.local != ember_mir::RETURN_LOCAL && aliases.contains(&place.local) && !matches!(rvalue, Rvalue::Aggregate { .. });
                if uses && !copied {
                    found.push((What::LetsThisOut, let_out.clone()));
                }
            }
            per_parameter(start, Point { block: at_block, index: at_stmt }, Point { block: at_block, index: at_stmt + 1 }, &mut found);
        }
        let Terminator::Call { func, args, next, .. } = &block.terminator else { continue };
        let start = found.len();
        let span = block.terminator_span;
        line.set(block.call_span.unwrap_or(span));
        op.set(match func {
            FuncRef::Builtin { which, .. } => builtin_op(which),
            _ => None,
        });
        let callees: Vec<usize> = match func {
            FuncRef::Direct { symbol, .. } => match index.get(symbol.as_str()) {
                Some(&callee) => vec![callee],
                // A function with no body here (C, the runtime): whatever its pointer arguments reach.
                None => {
                    for operand in args {
                        if let Operand::Copy(place) | Operand::Move(place) = operand
                            && points(types, place_type(body, types, place))
                        {
                            let mut through = place.clone();
                            through.projection.push(Projection::Deref);
                            own(What::Change, &through, &mut found);
                        }
                    }
                    Vec::new()
                }
            },
            FuncRef::Indirect { operand, ty, .. } => targets.of_call(at, body, operand, *ty, types),
            FuncRef::Virtual { owner, slot, .. } => bodies
                .iter()
                .enumerate()
                .filter(|(_, candidate)| candidate.class_virtual_slot == Some(*slot) && candidate.class_owner.is_some_and(|class| below(types, class, *owner)))
                .map(|(callee, _)| callee)
                .collect(),
            FuncRef::Interface { interfaces, slot, .. } => targets.of_dyn(interfaces, *slot).to_vec(),
            // A builtin changes what its `ref mut` arguments view and reads what its others view; one
            // that reshapes, what its first argument (both, for `mem.swap`) points at.
            FuncRef::Builtin { which, .. } => {
                if reshaping_builtin(types, which) {
                    let reshaped = if matches!(which, Builtin::MemSwap { .. }) { 2 } else { 1 };
                    for operand in args.iter().take(reshaped) {
                        if let Operand::Copy(place) | Operand::Move(place) = operand
                            && points(types, place_type(body, types, place))
                        {
                            let mut through = place.clone();
                            through.projection.push(Projection::Deref);
                            own(What::Reshape, &through, &mut found);
                            own(if adding_builtin(which) { What::Add } else { What::Remove }, &through, &mut found);
                        }
                    }
                }
                // `[DRP-8]` — a built-in that ends elements runs their `drop`s.
                if let Some((through, list)) = ended_elements(body, types, which, args) {
                    found.extend(ended_facts(body, types, bodies, targets, pointers, summaries, drop_runs, &through, list, span));
                }
                for operand in args {
                    if let Operand::Copy(place) | Operand::Move(place) = operand {
                        let ty = place_type(body, types, place);
                        let what = match types.kind(ty) {
                            TyKind::Ref { mutable: true, .. } | TyKind::Ptr { mutable: true, .. } => Some(What::Change),
                            TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Span { .. } | TyKind::Vec { .. } | TyKind::Class(_) | TyKind::Str | TyKind::CStr => Some(What::Read),
                            _ => None,
                        };
                        if let Some(what) = what {
                            let mut through = place.clone();
                            through.projection.push(Projection::Deref);
                            own(what, &through, &mut found);
                        }
                        own(What::Read, place, &mut found);
                    }
                }
                Vec::new()
            }
            _ => Vec::new(),
        };
        for operand in args {
            read(operand, &mut own, &mut found);
        }
        // The callee's facts, mapped onto the arguments: its parameter `i` is argument `i`.
        for &callee in &callees {
            let summary = &summaries[callee];
            for (what, facts) in [(What::Change, &summary.changes), (What::Read, &summary.reads), (What::Reshape, &summary.reshapes), (What::Add, &summary.adds), (What::Remove, &summary.removes), (What::End, &summary.ends)] {
                for fact in facts {
                    found.extend(mapped(body, types, pointers, args, callee, span, fact).into_iter().map(|fact| (what, fact)));
                }
            }
        }
        // `[CLS-2]` — `this` passed: to a method called on it, as that method lets it out; anywhere
        // else, let out here.
        for (position, argument) in args.iter().enumerate() {
            if !whole(argument) {
                continue;
            }
            let on_it = position == 0
                && matches!(func, FuncRef::Direct { .. } | FuncRef::Virtual { .. } | FuncRef::Interface { .. })
                && !callees.is_empty()
                && callees.iter().all(|&callee| is_method(&bodies[callee]));
            if !on_it {
                found.push((What::LetsThisOut, let_out.clone()));
                continue;
            }
            for &callee in &callees {
                if let Some(fact) = summaries[callee].lets_this_out.first() {
                    let mut chain = vec![(callee, span)];
                    chain.extend_from_slice(&fact.chain);
                    found.push((What::LetsThisOut, Fact { reach: let_out.reach.clone(), chain, at: fact.at, op: None }));
                }
            }
        }
        // `[THR-11]`, `[PAR-4]`, `[JOB-1]` (verdict 30) — a function a built-in runs itself (a scoped or
        // pooled task, a `@parallel` loop's chunks and combiner) does what its body does, from here.
        // `[EFF-1]` (review 4) — the code a built-in runs that the program wrote, with no call of its own (a
        // static's first-access initialiser, a `clone` a copy runs, the `Debug` a format runs, a function given
        // by value): what it does to a static, as it says; what it does through its parameters, to anything
        // this function's parameters reach.
        if matches!(func, FuncRef::Builtin { .. }) {
            let tasks: Vec<&str> = args.iter().filter_map(|operand| match operand {
                Operand::Const(Const::Fn(symbol)) => Some(symbol.as_str()),
                _ => None,
            }).collect();
            for run in builtin_runs(at, body, func, args, types, index, targets) {
                if tasks.contains(&bodies[run].symbol.as_str()) {
                    continue;
                }
                let summary = &summaries[run];
                for (what, facts) in [(What::Change, &summary.changes), (What::Read, &summary.reads), (What::Reshape, &summary.reshapes), (What::Add, &summary.adds), (What::Remove, &summary.removes), (What::End, &summary.ends)] {
                    for fact in facts {
                        let mut chain = vec![(run, span)];
                        chain.extend_from_slice(&fact.chain);
                        let mut out = Vec::new();
                        match &fact.reach.root {
                            Root::Static(..) => out.push(fact.reach.clone()),
                            Root::Param(_) => anything(body, &mut out),
                        }
                        found.extend(out.into_iter().map(|reach| (what, Fact { reach, chain: chain.clone(), at: fact.at, op: fact.op })));
                    }
                }
            }
        }
        if matches!(func, FuncRef::Builtin { .. }) {
            for operand in args {
                let Operand::Const(Const::Fn(symbol)) = operand else { continue };
                let Some(&task) = index.get(symbol.as_str()) else { continue };
                let summary = &summaries[task];
                for (what, facts) in [(What::Change, &summary.changes), (What::Read, &summary.reads), (What::Reshape, &summary.reshapes), (What::Add, &summary.adds), (What::Remove, &summary.removes), (What::End, &summary.ends)] {
                    for fact in facts {
                        found.extend(task_mapped(body, types, pointers, args, &bodies[task], task, span, fact).into_iter().map(|fact| (what, fact)));
                    }
                }
            }
        }
        let call = Point { block: at_block, index: block.stmts.len() };
        // What a call reads and changes is read or changed while a parameter is used when it is still
        // used after the call; one the call is its last use of is passed to the callee, whose own
        // facts about that parameter say what it does while using it (below).
        let after = Point { block: next.0 as usize, index: 0 };
        per_parameter(start, after, after, &mut found);
        // `[BRW-21]` — a view passed to a callee that may reshape or end a place before its last use of
        // that parameter, or read or change one while it uses it: so may this function, before its last
        // use of the parameter the view came from, or while it uses it.
        for &callee in &callees {
            let summary = &summaries[callee];
            let lists: [(&Vec<(usize, Fact)>, &[usize], fn(usize) -> What); 3] = [
                (&summary.used_after, &views, What::UsedAfter),
                (&summary.read_while, &unique, What::ReadWhile),
                (&summary.changed_while, &unique, What::ChangedWhile),
            ];
            for (facts, params, kind) in lists {
                for (param, fact) in facts {
                    let Some(origins) = args.get(*param).and_then(|argument| regions.and_then(|regions| regions.operand_origins_at(argument, call))) else { continue };
                    for origin in origins {
                        let Origin::Param(local) = origin else { continue };
                        let Some(view) = (local.0 as usize).checked_sub(1).filter(|view| params.contains(view)) else { continue };
                        for fact in mapped(body, types, pointers, args, callee, span, fact) {
                            if !through_itself(view, &fact.reach) {
                                found.push((kind(view), fact));
                            }
                        }
                    }
                }
            }
        }
    }
    found
}

/// A callee's fact, mapped onto a call's arguments: its parameter `i` is argument `i`, and a static is
/// the same place here; the call is the first step of the chain.
fn mapped(body: &Body, types: &TypeTable, pointers: &HashMap<LocalId, HashSet<Target>>, args: &[Operand], callee: usize, span: Span, fact: &Fact) -> Vec<Fact> {
    let chain = || {
        let mut chain = vec![(callee, span)];
        chain.extend_from_slice(&fact.chain);
        chain
    };
    let param = match &fact.reach.root {
        Root::Param(param) => *param,
        Root::Static(..) => return vec![Fact { reach: fact.reach.clone(), chain: chain(), at: fact.at, op: fact.op }],
    };
    let Some(Operand::Copy(place) | Operand::Move(place)) = args.get(param) else { return Vec::new() };
    let mut steps = steps_of(body, types, place);
    steps.extend_from_slice(&fact.reach.path);
    let mut out = Vec::new();
    reaches(body, pointers, &Spot { local: place.local, steps }, 0, &mut out);
    out.into_iter().map(|reach| Fact { reach, chain: chain(), at: fact.at, op: fact.op }).collect()
}

/// A fact of a function a built-in runs (`task`), from the call: its captured state is the call's
/// argument of the closure's environment type, each field the operand the environment was built from;
/// any other parameter may be anything the call's arguments point at; a static is the same place.
/// Where the environment cannot be followed, anything a parameter reaches.
#[allow(clippy::too_many_arguments)]
fn task_mapped(body: &Body, types: &TypeTable, pointers: &HashMap<LocalId, HashSet<Target>>, args: &[Operand], task_body: &Body, task: usize, span: Span, fact: &Fact) -> Vec<Fact> {
    let mut out = Vec::new();
    match (&fact.reach.root, task_body.closure_environment) {
        (Root::Static(..), _) => out.push(fact.reach.clone()),
        (Root::Param(0), Some(environment)) => {
            let place = args.iter().find_map(|operand| match operand {
                Operand::Copy(place) | Operand::Move(place) if matches!(types.kind(place_type(body, types, place)), TyKind::Struct(id) if *id == environment) => Some(place),
                _ => None,
            });
            // The task's state is its parameter, given by reference or by value: past the reference.
            let path = match fact.reach.path.split_first() {
                Some((Step::Deref, rest)) if matches!(task_body.locals.get(1).map(|local| types.kind(local.ty)), Some(TyKind::Ref { .. } | TyKind::Ptr { .. })) => rest,
                _ => &fact.reach.path[..],
            };
            match (place.and_then(|place| environment_fields(body, place, environment, 0)), path.split_first()) {
                (Some(fields), Some((Step::Field(field), rest))) => {
                    // A captured view: what the task does through it, it does to what the view points at;
                    // a captured value is the task's own.
                    if let Some(Operand::Copy(captured) | Operand::Move(captured)) = fields.get(*field) {
                        let mut steps = steps_of(body, types, captured);
                        steps.extend_from_slice(rest);
                        reaches(body, pointers, &Spot { local: captured.local, steps }, 0, &mut out);
                    }
                }
                (Some(_), _) => {}
                (None, _) => anything(body, &mut out),
            }
        }
        (Root::Param(_), _) => {
            for operand in args {
                if let Operand::Copy(place) | Operand::Move(place) = operand
                    && points(types, place_type(body, types, place))
                {
                    let mut steps = steps_of(body, types, place);
                    steps.push(Step::Deref);
                    reaches(body, pointers, &Spot { local: place.local, steps }, 0, &mut out);
                }
            }
        }
    }
    let mut chain = vec![(task, span)];
    chain.extend_from_slice(&fact.chain);
    out.into_iter().map(|reach| Fact { reach, chain: chain.clone(), at: fact.at, op: fact.op }).collect()
}

/// The operands a closure's environment held in `place` was built from, field by field: the struct
/// written into it (through copies of it); `None` when it is not one.
fn environment_fields(body: &Body, place: &Place, environment: ember_types::StructId, depth: usize) -> Option<Vec<Operand>> {
    if !place.projection.is_empty() || depth > 8 {
        return None;
    }
    let mut found = None;
    for stmt in body.blocks.iter().flat_map(|block| &block.stmts) {
        let StmtKind::Assign { place: written, rvalue } = &stmt.kind else { continue };
        if written.local != place.local || !written.projection.is_empty() {
            continue;
        }
        let fields = match rvalue {
            Rvalue::Aggregate { kind: ember_mir::AggregateKind::Struct(id), operands } if *id == environment => Some(operands.clone()),
            Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) => environment_fields(body, from, environment, depth + 1),
            _ => None,
        };
        // Written once, by one struct; any other write and it cannot be followed.
        match (fields, &found) {
            (Some(fields), None) => found = Some(fields),
            _ => return None,
        }
    }
    found
}

/// A method: its first parameter is its object, `this` (`self` in MIR).
fn is_method(body: &Body) -> bool {
    body.arg_count > 0 && body.locals.get(1).and_then(|local| local.name.as_deref()) == Some("self")
}

/// `[CLS-2]` — a method's locals holding its object as a whole: `this`, and each local given a copy
/// or a view of a whole one.
fn this_aliases(body: &Body) -> HashSet<LocalId> {
    let mut aliases = HashSet::new();
    if !is_method(body) {
        return aliases;
    }
    aliases.insert(LocalId(1));
    loop {
        let before = aliases.len();
        for stmt in body.blocks.iter().flat_map(|block| &block.stmts) {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            if !place.projection.is_empty() || place.local == ember_mir::RETURN_LOCAL {
                continue;
            }
            let source = match rvalue {
                Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) | Rvalue::Cast { operand: Operand::Copy(from) | Operand::Move(from), .. } | Rvalue::Ref { place: from, .. } => Some(from),
                _ => None,
            };
            if source.is_some_and(|from| whole_place(&aliases, from)) {
                aliases.insert(place.local);
            }
        }
        if aliases.len() == before {
            return aliases;
        }
    }
}

/// Whether `place` is a whole object one of `aliases` holds: the local, or what it points at.
fn whole_place(aliases: &HashSet<LocalId>, place: &Place) -> bool {
    aliases.contains(&place.local) && place.projection.iter().all(|projection| matches!(projection, Projection::Deref))
}

/// `[BRW-1]` — whether a fact's place is the parameter's own: reached through its own reference and
/// handle only (every `Deref` before its first field, element or variant step), not through a handle
/// or link stored in it.
fn through_itself(param: usize, reach: &Reach) -> bool {
    reach.root == Root::Param(param) && !reach.path.iter().skip_while(|step| **step == Step::Deref).any(|step| *step == Step::Deref)
}

/// Whether `class` is `owner` or below it.
fn below(types: &TypeTable, class: ember_types::ClassId, owner: ember_types::ClassId) -> bool {
    let mut current = Some(class);
    while let Some(at) = current {
        if at == owner {
            return true;
        }
        current = types.class_def(at).base;
    }
    false
}

/// `ember build --emit mir` — a body's summary, one line for each kind of fact: each place written
/// from the parameter's name and its fields' (`this.count`, `items[_]`), and the first callee of its
/// chain.
pub fn render(summary: &Summary, body: &Body, bodies: &[Body], types: &TypeTable) -> String {
    let place = |reach: &Reach| {
        // A method's object is `this` in its text (`[FN-4]`); a static is its name.
        let (mut text, mut ty) = match &reach.root {
            Root::Param(param) => {
                let local = body.locals.get(param + 1);
                let name = local.and_then(|local| local.name.clone()).map(|name| if name == "self" { "this".to_string() } else { name });
                (name.unwrap_or_else(|| format!("arg{param}")), local.map(|local| local.ty))
            }
            Root::Static(name, ty) => (name.rsplit('.').next().unwrap_or(name).to_string(), Some(*ty)),
        };
        let mut variant = None;
        for step in &reach.path {
            let current = ty.map(|ty| types.kind(ty).clone());
            let (shown, next) = match (step, current) {
                (Step::Deref, Some(TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. })) => (String::new(), Some(inner)),
                (Step::Deref, kind) => (String::new(), kind.and(ty)),
                (Step::Field(field) | Step::Column(field), Some(TyKind::Struct(id))) => match types.struct_def(id).fields.get(*field) {
                    Some(def) => (format!(".{}", def.name), Some(def.ty)),
                    None => (format!(".{field}"), None),
                },
                (Step::Field(field), Some(TyKind::Class(id))) => match types.class_field_at(id, *field) {
                    Some(def) => (format!(".{}", def.name), Some(def.ty)),
                    None => (format!(".{field}"), None),
                },
                (Step::Field(field), Some(TyKind::Enum(id))) => match variant.and_then(|variant: usize| types.enum_def(id).variants.get(variant)).and_then(|def| def.fields.get(*field)) {
                    Some(def) => (format!(".{}", def.name), Some(def.ty)),
                    None => (format!(".{field}"), None),
                },
                // A list's or a string's own fields (its length, its buffer) are the list.
                (Step::Field(_), Some(TyKind::Vec { .. })) => (String::new(), ty),
                (Step::Field(field) | Step::Column(field), _) => (format!(".{field}"), None),
                (Step::Variant(at), Some(TyKind::Enum(id))) => {
                    variant = Some(*at);
                    let name = types.enum_def(id).variants.get(*at).map_or_else(|| at.to_string(), |def| def.name.to_string());
                    (format!(" as {name}"), ty)
                }
                (Step::Variant(at), _) => (format!(" as {at}"), None),
                (Step::Elements, Some(TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } | TyKind::Span { elem, .. })) => ("[_]".to_string(), Some(elem)),
                (Step::Elements, _) => ("[_]".to_string(), None),
            };
            text.push_str(&shown);
            ty = next;
        }
        text
    };
    let line = |label: &str, facts: &[Fact]| {
        let mut items: Vec<String> = Vec::new();
        for fact in facts {
            let item = match fact.chain.first() {
                Some((callee, _)) => format!("{} (through {})", place(&fact.reach), bodies[*callee].name),
                None => place(&fact.reach),
            };
            if !items.contains(&item) {
                items.push(item);
            }
        }
        if items.is_empty() { String::new() } else { format!("  {label} {}\n", items.join(", ")) }
    };
    // `[EFF-24]` — of the places it adds to, those it only adds to.
    let only: Vec<Fact> = summary.adds.iter().filter(|fact| summary.only_adds(&fact.reach)).cloned().collect();
    // `[BRW-21]` — for each view parameter, what it may reshape or end before its last use of it.
    // `[BRW-1]` — and, for each `ref mut` one, what it reads and changes by other names while using it.
    let mut per_parameter = String::new();
    for (list, label) in [(&summary.used_after, "uses {} after it may reshape or end"), (&summary.read_while, "while it uses {}, reads"), (&summary.changed_while, "while it uses {}, changes")] {
        let mut params: Vec<usize> = list.iter().map(|(param, _)| *param).collect();
        params.dedup();
        for param in params {
            let facts: Vec<Fact> = list.iter().filter(|(at, _)| *at == param).map(|(_, fact)| fact.clone()).collect();
            let name = place(&Reach { root: Root::Param(param), path: Vec::new() });
            per_parameter.push_str(&line(&label.replace("{}", &name), &facts));
        }
    }
    // `[CLS-2]` — one way it lets `this` out is enough to say so.
    let lets_out = match summary.lets_this_out.first().map(|fact| fact.chain.first()) {
        Some(Some((callee, _))) => format!("  lets this out (through {})\n", bodies[*callee].name),
        Some(None) => "  lets this out\n".to_string(),
        None => String::new(),
    };
    format!(
        "{}{}{}{}{}{}{}{}",
        line("changes", &summary.changes),
        line("reads", &summary.reads),
        line("reshapes", &summary.reshapes),
        line("adds to", &summary.adds),
        line("only adds to", &only),
        line("ends", &summary.ends),
        per_parameter,
        lets_out
    )
}
