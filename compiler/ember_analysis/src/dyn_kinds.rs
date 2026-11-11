//! `[LT-1]` (verdict 151) — a call through a `dyn` value, or to a `virtual` method, is decided by the
//! code of each kind that may reach it: the types the program gives to the function, followed through
//! its names and calls.
//!
//! Inside a body, each value of a `dyn` type, an interface handle or a class handle is followed back to
//! where it is made (an upcast of a concrete value, a `Box[dyn I]`, a class constructor) or to the
//! parameter it came in by. A parameter holds what each call passes it, call by call, so a body is
//! checked once for each set of kinds its callers give it, as a generic is for each instance. A call
//! through such a value is then a call of the methods those kinds run, with their summaries joined
//! (`[LT-22]`); a set of more than one is a kind known only while running, and `E3021`'s message names
//! the kind that makes the line unsafe (`[LT-48]`). Where a value is not followed (a field, a list's
//! element, a result of a call through a value), the call keeps the worst case: its result points into
//! every argument. A virtual call on a class value not followed may run the body of any class at or
//! below its class in the build (`[DSP-5]`). See `tasks/96/redesign/compiler-0.9.15/dyn-per-kind-design.md`.

use std::collections::{BTreeSet, HashMap};

use ember_hir::Builtin;
use ember_mir::{
    Body, CallableAccessSummary as CallAccessContract, CastKind, FuncRef, LocalId, Operand,
    ParameterFieldAccess, Place, Projection, ResultRegionSource, Rvalue, StmtKind, Terminator,
};
use ember_span::Symbol;
use ember_types::{ClassId, ClassOpenness, Ty, TyKind, TypeTable};

use crate::regions::{project_type, CallRegionContract, CallResultContract, Elision};

/// At most this many sets of kinds a body is checked for.
// ponytail: a fixed cap; past it, a body's further callers are checked as one, with the worst case for
// its parameters. Raise it, or merge the sets, if a real program needs more.
const MOST_INSTANCES: usize = 8;

/// Where a value may come from: a parameter of its body, a kind it is made as, or a place not followed.
/// A body's instance holds only kinds and `Unknown`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Origin {
    Param(usize),
    Kind(usize),
    Unknown,
}

type Origins = BTreeSet<Origin>;

/// One kind a value may be.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Kind {
    /// A concrete type behind the table of `interfaces`: the method each slot holds. A class concrete's
    /// object may be of a class below it, whose override the table runs.
    Table { concrete: Ty, name: String, interfaces: Vec<Symbol>, implementations: Vec<Option<String>>, class: Option<ClassId> },
    /// An object made as exactly this class.
    Class(ClassId),
}

/// A method a call may run, and the kind (by its name) that runs it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Target {
    pub name: String,
    pub symbol: String,
    /// The method's name, as the program writes it (`pick`).
    pub method: String,
    /// `[LT-48]` — the kind, where a value's table tells it (`Table`): a concrete that is not a class.
    pub kind: Option<usize>,
}

/// `[LT-48]` — the table an upcast of `concrete` to `interfaces` makes, which a `runtime`
/// parameter's check compares its value's table with (`CastKind::DynKindIs`).
#[derive(Clone, Debug)]
pub(crate) struct Table {
    pub concrete: Ty,
    pub interfaces: Vec<Symbol>,
    pub layout: Vec<Option<ember_hir::InterfaceSlot>>,
    pub implementations: Vec<Option<ember_mir::InterfaceAdapterMethod>>,
}

/// `[LT-48]` — a check of a `runtime` parameter's kind, before the line a kind makes unsafe: at
/// that point, the value of `param` is tested for `table`'s kind, and the program stops with
/// `message` when it is that kind.
#[derive(Clone, Debug)]
pub(crate) struct Guard {
    pub block: usize,
    pub index: usize,
    pub span: ember_span::Span,
    pub param: LocalId,
    pub table: Table,
    pub message: String,
}

/// A `dyn` or virtual call of a body, in one of the forms it is checked in.
#[derive(Clone, Debug, Default)]
pub(crate) struct DynSite {
    /// The methods it may run; empty where its receiver is not followed (every argument, the worst case).
    pub targets: Vec<Target>,
    /// For each argument, the methods (by place in `targets`) whose result points into it.
    pub ties: Vec<Vec<usize>>,
    /// `[LT-48]` — the help, where the receiver is a `dyn` parameter whose kind is known only while running.
    pub help: Option<String>,
    /// `[LT-48]` — the parameter written `runtime` the receiver comes from, where its kind is known
    /// only while running; and each target's table, where its kind can be tested.
    pub runtime: Option<LocalId>,
    pub tables: Vec<Option<Table>>,
    /// For each argument, what a method whose result points into it gives back ("the list it was
    /// handed").
    pub handed: Vec<String>,
}

/// A body's `dyn` and virtual calls in one form, by block.
pub(crate) type DynSites = HashMap<usize, DynSite>;

/// One form a body is checked in: a copy with its resolved calls made direct, or (`None`) the body;
/// and the checks of a `runtime` parameter's kind its check asks for (`[LT-48]`).
#[derive(Default)]
pub(crate) struct Form {
    pub body: Option<Body>,
    pub sites: DynSites,
    pub guards: std::sync::Mutex<Vec<Guard>>,
}

/// What [`expand`] finds.
#[derive(Default)]
pub(crate) struct Expansion {
    /// By body index, each form to check in place of the body. A body not here is checked as it is.
    pub forms: HashMap<usize, Vec<Form>>,
    /// The contract of each symbol a form's call is made direct to: a set of methods a `dyn` or
    /// virtual call may run (their summaries joined), or another body's form (its summary).
    pub contracts: HashMap<String, CallRegionContract>,
    /// `[BLD-1]` — by body index, what its forms' checks read: each form's calls made direct, with
    /// what they were made direct to and the summary of each. A kept body is checked again when its
    /// record kept other ones (`BorrowFacts::kinds`).
    pub checked_under: HashMap<usize, Vec<String>>,
}

/// How many times the forms are made again, each with the callers of a form whose summary is not its
/// body's made direct to it.
// ponytail: a fixed bound on the chain of callers given a form's summary; past it, a caller reads the
// body's summary (the worst case for its `dyn` calls), which is sound.
const MOST_ROUNDS: usize = 16;

/// `[LT-1]`, `[LT-48]` — the forms each body with a `dyn` or virtual call is checked in, and each of
/// its callers that reads its summary (`[LT-22]`): a call in a caller's form is made direct to the
/// form of the instance it gives its callee, where that form's summary is not the callee's own.
/// `contract` gives a body's summary by its symbol; `infer` infers the summaries of copies of bodies,
/// given the summaries of further symbols they call.
pub(crate) fn expand(
    bodies: &[Body],
    types: &TypeTable,
    contract: &dyn Fn(&str) -> CallRegionContract,
    infer: &dyn Fn(&[Body], &HashMap<String, CallRegionContract>) -> HashMap<String, CallRegionContract>,
) -> Expansion {
    let sites: Vec<Vec<usize>> = bodies
        .iter()
        .map(|body| {
            (0..body.blocks.len())
                .filter(|&block| {
                    matches!(&body.blocks[block].terminator, Terminator::Call { func: FuncRef::Interface { .. } | FuncRef::Virtual { .. }, args, .. } if !args.is_empty())
                })
                .collect()
        })
        .collect();
    if sites.iter().all(Vec::is_empty) {
        return Expansion::default();
    }
    let mut kinds = Kinds::new(bodies, types);
    kinds.follow();
    let (instances, calls) = kinds.instances();
    // Each instance's `dyn` and virtual calls, resolved, and each set of methods' joined contract.
    let resolved: Vec<Vec<Vec<Resolved>>> = (0..bodies.len())
        .map(|at| {
            instances[at]
                .iter()
                .map(|instance| {
                    sites[at]
                        .iter()
                        .map(|&block| {
                            let (targets, help) = kinds.resolve(at, instance, block);
                            (block, targets, help)
                        })
                        .collect()
                })
                .collect()
        })
        .collect();
    let mut joined: HashMap<String, CallRegionContract> = HashMap::new();
    for (_, targets, _) in resolved.iter().flatten().flatten() {
        if !targets.is_empty() {
            joined
                .entry(joined_symbol(targets))
                .or_insert_with(|| join(targets_symbols(targets).iter().map(|symbol| contract(symbol)).collect()));
        }
    }
    // The calls of each instance made direct to their callee's form: more each round, never fewer.
    let mut needs: BTreeSet<(usize, usize, usize)> = BTreeSet::new();
    let mut made = Made::default();
    for _ in 0..MOST_ROUNDS {
        made = Made::new(bodies, &resolved, &calls, &needs);
        made.summaries = infer(&made.bodies, &joined);
        let mut more = needs.clone();
        for (at, per_instance) in calls.iter().enumerate() {
            for (which, calls) in per_instance.iter().enumerate() {
                for &(block, callee, given) in calls {
                    if let Some(symbol) = &made.symbol_of[callee][given]
                        && made.summaries.get(symbol).is_some_and(|summary| *summary != contract(&bodies[callee].symbol))
                    {
                        more.insert((at, which, block));
                    }
                }
            }
        }
        if more == needs {
            break;
        }
        needs = more;
    }
    // Each body's forms: one per copy made of it, and the body itself for an instance resolving
    // nothing (one per different set of helps).
    let mut chosen: Vec<(usize, Vec<(Option<String>, usize)>)> = Vec::new();
    for at in 0..bodies.len() {
        if sites[at].is_empty() && !made.symbol_of[at].iter().any(Option::is_some) {
            continue;
        }
        let mut forms: Vec<(Option<String>, usize)> = Vec::new();
        for (which, symbol) in made.symbol_of[at].iter().enumerate() {
            let seen = forms.iter().any(|(other, first)| other == symbol && (symbol.is_some() || resolved[at][*first] == resolved[at][which]));
            if !seen {
                forms.push((symbol.clone(), which));
            }
        }
        chosen.push((at, forms));
    }
    let mut copies: HashMap<String, Body> = std::mem::take(&mut made.bodies).into_iter().map(|body| (body.symbol.clone(), body)).collect();
    let mut expansion = Expansion::default();
    for (at, forms) in chosen {
        let mut under = Vec::new();
        let forms = forms
            .into_iter()
            .map(|(symbol, which)| {
                let mut sites = DynSites::new();
                let mut read = Vec::new();
                for (block, targets, help) in &resolved[at][which] {
                    let Terminator::Call { args, .. } = &bodies[at].blocks[*block].terminator else { continue };
                    let ties = if targets.is_empty() {
                        read.push(format!("{block}:*"));
                        Vec::new()
                    } else {
                        let each: Vec<String> = targets
                            .iter()
                            .map(|target| format!("{}={}", target.symbol, digest(&format!("{:?}", contract(&target.symbol)))))
                            .collect();
                        read.push(format!("{block}:{}", each.join(",")));
                        (0..args.len())
                            .map(|argument| {
                                (0..targets.len()).filter(|&target| whole_receiver(contract(&targets[target].symbol)).ties(argument)).collect()
                            })
                            .collect()
                    };
                    let gives = args
                        .iter()
                        .map(|arg| match arg {
                            Operand::Copy(place) | Operand::Move(place) => handed(types, crate::regions::place_type(&bodies[at], types, place)),
                            Operand::Const(_) => "the value it was handed".to_string(),
                        })
                        .collect();
                    sites.insert(
                        *block,
                        DynSite {
                            targets: targets.clone(),
                            ties,
                            help: help.clone(),
                            runtime: kinds.runtime_param(at, &instances[at][which], *block),
                            tables: targets.iter().map(|target| target.kind.and_then(|kind| kinds.tables[kind].clone())).collect(),
                            handed: gives,
                        },
                    );
                }
                for &(block, callee, given) in &calls[at][which] {
                    if needs.contains(&(at, which, block))
                        && let Some(form) = &made.symbol_of[callee][given]
                    {
                        read.push(format!("{block}->{form}={}", digest(&format!("{:?}", made.summaries.get(form)))));
                    }
                }
                under.push(read.join(";"));
                Form { body: symbol.and_then(|symbol| copies.remove(&symbol)), sites, guards: Default::default() }
            })
            .collect();
        under.sort();
        expansion.checked_under.insert(at, under);
        expansion.forms.insert(at, forms);
    }
    expansion.contracts = joined;
    expansion.contracts.extend(made.summaries);
    expansion
}

/// A `dyn` or virtual call of an instance: its block, the methods it may run, and the help.
type Resolved = (usize, Vec<Target>, Option<String>);

/// The copies of the bodies one round makes: for each instance whose `dyn` or virtual calls resolve,
/// or whose calls are made direct to a callee's form, a copy (one per different form, by a symbol of
/// its own); and their summaries, once inferred.
#[derive(Default)]
struct Made {
    bodies: Vec<Body>,
    /// By body and instance, the symbol of the copy its instance is checked in; `None`: the body.
    symbol_of: Vec<Vec<Option<String>>>,
    summaries: HashMap<String, CallRegionContract>,
}

impl Made {
    fn new(bodies: &[Body], resolved: &[Vec<Vec<Resolved>>], calls: &[Vec<Vec<(usize, usize, usize)>>], needs: &BTreeSet<(usize, usize, usize)>) -> Made {
        // A form's name holds what its calls are made direct to, its callees' forms by their names: to
        // a fixed point, from the forms that resolve their own calls.
        let own: Vec<Vec<Option<String>>> = resolved
            .iter()
            .map(|per_instance| {
                per_instance
                    .iter()
                    .map(|sites| {
                        let parts: Vec<String> = sites
                            .iter()
                            .filter(|(_, targets, _)| !targets.is_empty())
                            .map(|(block, targets, help)| format!("{block}:{}|{}", joined_symbol(targets), help.as_deref().unwrap_or("-")))
                            .collect();
                        (!parts.is_empty()).then(|| parts.join(";"))
                    })
                    .collect()
            })
            .collect();
        let mut keys = own.clone();
        for _ in 0..=bodies.len() {
            let mut changed = false;
            for at in 0..bodies.len() {
                for which in 0..keys[at].len() {
                    let mut parts: Vec<String> = own[at][which].iter().cloned().collect();
                    for &(block, callee, given) in &calls[at][which] {
                        if needs.contains(&(at, which, block))
                            && let Some(key) = &keys[callee][given]
                        {
                            parts.push(format!("{block}->{}", form_symbol(&bodies[callee].symbol, key)));
                        }
                    }
                    let key = (!parts.is_empty()).then(|| parts.join(";"));
                    if key != keys[at][which] {
                        keys[at][which] = key;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let symbol_of: Vec<Vec<Option<String>>> = (0..bodies.len())
            .map(|at| keys[at].iter().map(|key| key.as_ref().map(|key| form_symbol(&bodies[at].symbol, key))).collect())
            .collect();
        let mut copies = Vec::new();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for at in 0..bodies.len() {
            for (which, symbol) in symbol_of[at].iter().enumerate() {
                let Some(symbol) = symbol else { continue };
                if !seen.insert(symbol) {
                    continue;
                }
                let mut copy = bodies[at].clone();
                copy.symbol = symbol.clone();
                for (block, targets, _) in &resolved[at][which] {
                    if !targets.is_empty()
                        && let Terminator::Call { func, .. } = &mut copy.blocks[*block].terminator
                    {
                        *func = FuncRef::Direct { symbol: joined_symbol(targets), latebound: false };
                    }
                }
                for &(block, callee, given) in &calls[at][which] {
                    if needs.contains(&(at, which, block))
                        && let Some(form) = &symbol_of[callee][given]
                        && let Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } = &mut copy.blocks[block].terminator
                    {
                        *symbol = form.clone();
                    }
                }
                copies.push(copy);
            }
        }
        Made { bodies: copies, symbol_of, summaries: HashMap::new() }
    }
}

/// The symbol a call that may run `targets` is made direct to.
fn joined_symbol(targets: &[Target]) -> String {
    format!("\u{1}dyn {}", targets_symbols(targets).join(" | "))
}

/// Whether `symbol` is one a `dyn` or virtual call is made direct to (a method call, as the call it
/// replaces), not a body's form.
pub(crate) fn is_joined(symbol: &str) -> bool {
    symbol.starts_with("\u{1}dyn ")
}

fn targets_symbols(targets: &[Target]) -> Vec<&str> {
    targets.iter().map(|target| target.symbol.as_str()).collect::<BTreeSet<_>>().into_iter().collect()
}

/// The symbol of the form of the body `symbol` that `key` describes.
fn form_symbol(symbol: &str, key: &str) -> String {
    format!("{symbol}\u{1}kinds {}", digest(key))
}

/// A short name of `text`, the same in every run of one compiler.
fn digest(text: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// `[LT-1]`, `[LT-48]` — a loan held by an argument of a `dyn` or virtual call of a form.
pub(crate) struct Explained<'s> {
    pub block: usize,
    pub argument: usize,
    pub site: &'s DynSite,
    /// The kinds (by name) whose method's result points into the argument, when they are some but
    /// not all of the kinds the call may run.
    pub culprits: Option<Vec<&'s str>>,
}

/// `[LT-1]`, `[LT-48]` — where the loan `borrower` holds is an argument of a `dyn` or virtual call of
/// `sites`: the call's block and site, and the kinds that make a change of what it holds unsafe.
pub(crate) fn explain<'s>(body: &Body, sites: &'s DynSites, borrower: LocalId) -> Option<Explained<'s>> {
    let borrower = converted(body, sites, borrower);
    let mut found: Vec<(usize, &DynSite, usize)> = sites
        .iter()
        .filter_map(|(&block, site)| {
            let Terminator::Call { args, .. } = &body.blocks[block].terminator else { return None };
            let argument = args.iter().position(|arg| matches!(arg, Operand::Copy(place) | Operand::Move(place) if place.local == borrower))?;
            Some((block, site, argument))
        })
        .collect();
    found.sort_by_key(|(block, ..)| *block);
    let (block, site, argument) = found.into_iter().next()?;
    // Only a loan nothing but the call's argument holds is kept by the call's result: a view named
    // and used again (`v = ref xs`, `p.pick(v)`, `println(r, v)`) keeps it whatever the kind.
    if !only_the_call_reads(body, borrower, block) {
        return None;
    }
    let kinds: BTreeSet<&str> = site.targets.iter().map(|target| target.name.as_str()).collect();
    let mut names: Vec<&str> = site.ties.get(argument).into_iter().flatten().map(|&target| site.targets[target].name.as_str()).collect();
    names.dedup();
    let culprits = (!names.is_empty() && names.len() < kinds.len()).then_some(names);
    (culprits.is_some() || site.help.is_some()).then_some(Explained { block, argument, site, culprits })
}

/// `[LT-48]` (D-967) — the value a conversion makes of `local` for the call: an argument given as
/// another view type is converted first (a `String` where a `StrView` is wanted: `_7 = &s`,
/// `_6 = as_str(copy _7)`, then `p.title(_6)`), so the loan `_7` holds reaches the `dyn` call
/// through `_6`. Followed while `local` is a temporary only a call of it alone reads, and that call's
/// result a temporary and not itself a `dyn` call; any view type is converted so, `Span` and text alike.
fn converted(body: &Body, sites: &DynSites, mut local: LocalId) -> LocalId {
    for _ in 0..body.blocks.len() {
        let next = body.blocks.iter().enumerate().find_map(|(at, basic)| {
            let Terminator::Call { args, dest, .. } = &basic.terminator else { return None };
            let [Operand::Copy(place) | Operand::Move(place)] = args.as_slice() else { return None };
            (place.local == local
                && place.projection.is_empty()
                && dest.projection.is_empty()
                && !sites.contains_key(&at)
                && body.local(dest.local).kind == ember_mir::LocalKind::Temp
                && only_the_call_reads(body, local, at))
                .then_some(dest.local)
        });
        match next {
            Some(next) if next != local => local = next,
            _ => break,
        }
    }
    local
}

/// Whether `local` is a temporary that nothing reads but the call at `block`.
fn only_the_call_reads(body: &Body, local: LocalId, block: usize) -> bool {
    if body.local(local).kind != ember_mir::LocalKind::Temp {
        return false;
    }
    let reads = |operand: &Operand| matches!(operand, Operand::Copy(place) | Operand::Move(place) if place.local == local);
    let names = |place: &Place| place.local == local;
    for (at, basic) in body.blocks.iter().enumerate() {
        for stmt in &basic.stmts {
            let read = match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    (names(place) && !place.projection.is_empty())
                        || match rvalue {
                            Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } | Rvalue::Repeat { value: operand, .. } => reads(operand),
                            Rvalue::BinaryOp { lhs, rhs, .. } => reads(lhs) || reads(rhs),
                            Rvalue::Aggregate { operands, .. } => operands.iter().any(reads),
                            Rvalue::Ref { place, .. } | Rvalue::Discriminant(place) => names(place),
                        }
                }
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => reads(lhs) || reads(rhs),
                StmtKind::BeginAccess { place, .. }
                | StmtKind::BeginAccessTransfer { place, .. }
                | StmtKind::EndAccess { place, .. }
                | StmtKind::EndAccessTransfer { place, .. }
                | StmtKind::LendAccess { place, .. }
                | StmtKind::Drop { place, .. } => names(place),
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => false,
            };
            if read {
                return false;
            }
        }
        let read = match &basic.terminator {
            Terminator::Call { func, args, .. } => at != block && (args.iter().any(reads) || matches!(func, FuncRef::Indirect { operand, .. } if reads(operand))),
            Terminator::SwitchInt { discr, .. } => reads(discr),
            Terminator::Assert { cond, .. } => reads(cond),
            _ => false,
        };
        if read {
            return false;
        }
    }
    true
}

/// `[LT-48]` — the checks that stand for an `E3021` of `explained`'s line, where its receiver is a
/// `runtime` parameter: one for each kind that makes the line unsafe, before the line (`at`). `None`
/// where one of those kinds cannot be told by its table (a class's override), or the line has no
/// name holding the view (`keeper`).
pub(crate) fn guards(body: &Body, explained: &Explained, changed: &str, keeper: Option<&str>, at: (usize, usize), span: ember_span::Span) -> Option<Vec<Guard>> {
    let site = explained.site;
    let param = site.runtime?;
    explained.culprits.as_ref()?;
    let keeper = keeper?;
    let receiver = body.local(param).name.as_deref()?;
    site.ties
        .get(explained.argument)?
        .iter()
        .map(|&target| {
            let kind = &site.targets[target];
            let table = site.tables.get(target)?.clone()?;
            let gives = if explained.argument == 0 { "what it holds".to_string() } else { site.handed.get(explained.argument)?.clone() };
            Some(Guard {
                block: at.0,
                index: at.1,
                span,
                param,
                table,
                message: format!(
                    "{changed} changes while `{keeper}` looks at it: `{receiver}` is {}, whose `{}` gives back {gives}",
                    kinds_named(&[kind.name.as_str()]),
                    kind.method
                ),
            })
        })
        .collect()
}

/// What a method gives back that points into an argument of type `ty`: "the list it was handed".
fn handed(types: &TypeTable, mut ty: Ty) -> String {
    while let TyKind::Ref { inner, .. } = types.kind(ty) {
        ty = *inner;
    }
    let noun = match types.kind(ty) {
        // D-968: a `String` is a `Vec` marked as text; it is named as the text it is.
        TyKind::Str | TyKind::Vec { text: true, .. } => "text".to_string(),
        TyKind::Array { .. } | TyKind::ArrayN { .. } | TyKind::Vec { .. } | TyKind::Span { .. } => "list".to_string(),
        _ => format!("`{}`", types.display(ty)),
    };
    format!("the {noun} it was handed")
}

/// The name of the local the receiver of the call at `block` comes from, through the temporaries
/// that copy or lend it (`_5 = &a`, `_12 = &(*b.0)`).
pub(crate) fn receiver_name(body: &Body, block: usize) -> Option<&str> {
    let Terminator::Call { args, .. } = &body.blocks[block].terminator else { return None };
    let (Operand::Copy(place) | Operand::Move(place)) = args.first()? else { return None };
    let mut local = place.local;
    for _ in 0..body.locals.len() {
        if let Some(name) = body.local(local).name.as_deref() {
            return Some(name);
        }
        local = body.blocks.iter().flat_map(|block| &block.stmts).find_map(|stmt| match &stmt.kind {
            StmtKind::Assign { place, rvalue: Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) | Rvalue::Ref { place: from, .. } | Rvalue::Cast { operand: Operand::Copy(from) | Operand::Move(from), .. } }
                if place.local == local && place.projection.is_empty() =>
            {
                Some(from.local)
            }
            _ => None,
        })?;
    }
    None
}

/// `[LT-1]` — the kinds a line is unsafe for, as `E3021` names them: "a `Yours`", "a `Yours` or an `Item`".
pub(crate) fn kinds_named(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| {
            let article = if name.starts_with(['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
            format!("{article} `{name}`")
        })
        .collect::<Vec<_>>()
        .join(" or ")
}

/// The contract of a call that runs one of several methods: what each one's result points into and
/// what each touches, together; the receiver's part is the whole receiver ([`whole_receiver`]).
fn join(contracts: Vec<CallRegionContract>) -> CallRegionContract {
    let mut contracts = contracts.into_iter().map(whole_receiver);
    let Some(mut joined) = contracts.next() else {
        return CallRegionContract {
            access: CallAccessContract::All,
            result: CallResultContract::Legacy(Elision::Everything),
            latebound: false,
            stores: Default::default(),
        };
    };
    for next in contracts {
        joined.access = match (joined.access, next.access) {
            (CallAccessContract::Fields(mut fields), CallAccessContract::Fields(more)) => {
                for access in more {
                    match fields.iter_mut().find(|field| field.argument == access.argument && field.projection == access.projection) {
                        Some(field) => {
                            field.operations.extend(access.operations);
                            field.operations.sort();
                            field.operations.dedup();
                        }
                        None => fields.push(access),
                    }
                }
                CallAccessContract::Fields(fields)
            }
            _ => CallAccessContract::All,
        };
        joined.result = match (joined.result, next.result) {
            (one, other) if one == other => one,
            (CallResultContract::Fields(mut one), CallResultContract::Fields(other))
                if one.fields.len() == other.fields.len()
                    && one.fields.iter().zip(&other.fields).all(|(a, b)| a.result_projection == b.result_projection) =>
            {
                for (field, more) in one.fields.iter_mut().zip(other.fields) {
                    field.sources.extend(more.sources);
                    field.sources.sort();
                    field.sources.dedup();
                }
                CallResultContract::Fields(one)
            }
            // Results of different shapes: the worst case, as before.
            _ => CallResultContract::Legacy(Elision::Everything),
        };
    }
    joined.latebound = false;
    // A method a table or a virtual slot holds answers for its stores itself (`stores.rs`).
    joined.stores = Default::default();
    joined
}

/// A method's summary as a `dyn` or virtual call reads it: its receiver's parts (paths in the method's
/// own type, `this.items` of `Mine`) are the whole receiver, which the call's receiver (a `ref dyn I`,
/// a base class) is. The worst case says no less of the receiver.
fn whole_receiver(mut contract: CallRegionContract) -> CallRegionContract {
    if let CallAccessContract::Fields(fields) = &mut contract.access {
        let mut operations = Vec::new();
        fields.retain(|field| {
            if field.argument == 0 {
                operations.extend(field.operations.iter().copied());
            }
            field.argument != 0
        });
        if !operations.is_empty() {
            operations.sort();
            operations.dedup();
            fields.insert(0, ParameterFieldAccess { argument: 0, projection: Vec::new(), operations });
        }
    }
    match &mut contract.result {
        CallResultContract::Fields(summary) => {
            for field in &mut summary.fields {
                for source in &mut field.sources {
                    if let ResultRegionSource::View { argument: 0, projection } = source {
                        projection.clear();
                    }
                }
                field.sources.sort();
                field.sources.dedup();
            }
        }
        CallResultContract::Legacy(Elision::Named { arguments, through }) => {
            if through.iter().any(|(argument, _)| *argument == 0) && !arguments.contains(&0) {
                arguments.push(0);
            }
            through.retain(|(argument, _)| *argument != 0);
        }
        CallResultContract::Legacy(_) => {}
    }
    contract
}

/// Whether a value of `ty` carries a kind: a `dyn`, a `Box[dyn I]`, an interface or class handle, or a
/// reference to one.
fn carrier(types: &TypeTable, ty: Ty) -> bool {
    match types.kind(ty) {
        TyKind::Dyn { .. } | TyKind::ClassInterface(_) | TyKind::Class(_) => true,
        TyKind::Ref { inner, .. } => carrier(types, *inner),
        TyKind::Struct(id) => types.compiler_box_inner(*id).is_some_and(|inner| matches!(types.kind(inner), TyKind::Dyn { .. })),
        _ => false,
    }
}

struct Kinds<'a> {
    bodies: &'a [Body],
    types: &'a TypeTable,
    index: HashMap<&'a str, usize>,
    kinds: Vec<Kind>,
    ids: HashMap<Kind, usize>,
    /// By kind, its table where a value's table tells it (`[LT-48]`).
    tables: Vec<Option<Table>>,
    /// Per body, what each local of a kind-carrying type may hold.
    origins: Vec<HashMap<LocalId, Origins>>,
    /// Per body whose result carries a kind, what it returns.
    returns: Vec<Option<Origins>>,
    /// Each class's own method in each virtual slot.
    defined: HashMap<(ClassId, usize), usize>,
    /// Each table's flattened layout, by its interfaces, as the program's upcasts carry it.
    layouts: HashMap<Vec<Symbol>, Vec<Option<ember_hir::InterfaceSlot>>>,
}

impl<'a> Kinds<'a> {
    fn new(bodies: &'a [Body], types: &'a TypeTable) -> Kinds<'a> {
        let defined = bodies
            .iter()
            .enumerate()
            .filter(|(_, body)| !body.is_abstract)
            .filter_map(|(at, body)| Some(((body.class_owner?, body.class_virtual_slot?), at)))
            .collect();
        let mut layouts = HashMap::new();
        for body in bodies {
            for block in &body.blocks {
                for stmt in &block.stmts {
                    if let StmtKind::Assign { rvalue: Rvalue::Cast { kind: CastKind::InterfaceUpcast { interfaces, layout, .. }, .. }, .. } = &stmt.kind {
                        layouts.entry(interfaces.clone()).or_insert_with(|| layout.clone());
                    }
                }
                if let Terminator::Call { func: FuncRef::DynBoxNew { interfaces, layout, .. }, .. } = &block.terminator {
                    layouts.entry(interfaces.clone()).or_insert_with(|| layout.clone());
                }
            }
        }
        Kinds {
            layouts,
            bodies,
            types,
            index: bodies.iter().enumerate().map(|(at, body)| (body.symbol.as_str(), at)).collect(),
            kinds: Vec::new(),
            ids: HashMap::new(),
            tables: Vec::new(),
            origins: vec![HashMap::new(); bodies.len()],
            returns: bodies.iter().map(|body| carrier(types, body.return_ty()).then(Origins::new)).collect(),
            defined,
        }
    }

    fn kind(&mut self, kind: Kind) -> Origin {
        let next = self.kinds.len();
        let id = *self.ids.entry(kind.clone()).or_insert(next);
        if id == next {
            self.kinds.push(kind);
            self.tables.push(None);
        }
        Origin::Kind(id)
    }

    fn table(&mut self, concrete: Ty, interfaces: &[Symbol], implementations: &[Option<ember_mir::InterfaceAdapterMethod>]) -> Origin {
        let class = match self.types.kind(concrete) {
            TyKind::Class(id) => Some(*id),
            _ => None,
        };
        let origin = self.kind(Kind::Table {
            concrete,
            name: self.types.display(concrete),
            interfaces: interfaces.to_vec(),
            implementations: implementations.iter().map(|method| method.as_ref().map(|method| method.symbol.clone())).collect(),
            class,
        });
        if let Origin::Kind(kind) = origin
            && class.is_none()
            && self.tables[kind].is_none()
        {
            let layout = self.types_layout(interfaces);
            self.tables[kind] = Some(Table { concrete, interfaces: interfaces.to_vec(), layout, implementations: implementations.to_vec() });
        }
        origin
    }

    /// Each body's origins, and what each returns, to a fixed point: a body's result is what its
    /// callees return, their parameters replaced by what it passes them.
    fn follow(&mut self) {
        let bodies = self.bodies;
        let relevant: Vec<usize> = (0..bodies.len()).filter(|&at| bodies[at].locals.iter().any(|local| carrier(self.types, local.ty))).collect();
        for _ in 0..=bodies.len() {
            let mut changed = false;
            for &at in &relevant {
                let origins = self.body_origins(at);
                if let Some(returned) = &self.returns[at] {
                    let now = origins.get(&ember_mir::RETURN_LOCAL).cloned().unwrap_or_default();
                    if &now != returned {
                        self.returns[at] = Some(now);
                        changed = true;
                    }
                }
                self.origins[at] = origins;
            }
            if !changed {
                return;
            }
        }
        // Not settled in the rounds given: what a body returns is not followed.
        for returned in self.returns.iter_mut().flatten() {
            *returned = Origins::from([Origin::Unknown]);
        }
        for &at in &relevant {
            self.origins[at] = self.body_origins(at);
        }
    }

    fn body_origins(&mut self, at: usize) -> HashMap<LocalId, Origins> {
        let bodies = self.bodies;
        let types = self.types;
        let body = &bodies[at];
        let mut out: HashMap<LocalId, Origins> = HashMap::new();
        for (id, local) in body.locals.iter().enumerate() {
            if carrier(types, local.ty) {
                out.insert(LocalId(id as u32), Origins::new());
            }
        }
        for (local, _) in body.args() {
            if let Some(set) = out.get_mut(&local) {
                set.insert(Origin::Param(local.0 as usize - 1));
            }
        }
        // A value whose kind may be replaced through a view of it is not followed: one a `ref mut`
        // is taken of (`mem.swap(ref mut a, ref mut b)`, a callee's `ref mut` parameter, a closure's
        // capture), and one written through a reference or the box holding it (`*q = Box(…)`). A
        // `ref mut` of the `dyn` value itself (a `ref mut this` call) cannot replace it.
        let mut replaced: Vec<LocalId> = Vec::new();
        for block in &body.blocks {
            for stmt in &block.stmts {
                let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
                if let Rvalue::Ref { place: lent, mutable: true } = rvalue
                    && out.contains_key(&lent.local)
                    && transparent(body, types, lent)
                    && !matches!(types.kind(crate::regions::place_type(body, types, lent)), TyKind::Dyn { .. })
                {
                    replaced.push(lent.local);
                }
                if !place.projection.is_empty() && out.contains_key(&place.local) && transparent(body, types, place) {
                    replaced.push(place.local);
                }
            }
            if let Terminator::Call { dest, .. } = &block.terminator
                && !dest.projection.is_empty()
                && out.contains_key(&dest.local)
                && transparent(body, types, dest)
            {
                replaced.push(dest.local);
            }
        }
        for local in replaced {
            out.entry(local).or_default().insert(Origin::Unknown);
        }
        loop {
            let mut changed = false;
            for block in &body.blocks {
                for stmt in &block.stmts {
                    let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
                    if !place.projection.is_empty() || !out.contains_key(&place.local) {
                        continue;
                    }
                    let found = match rvalue {
                        Rvalue::Use(operand) => of_operand(body, types, &out, operand),
                        Rvalue::Ref { place: lent, .. } => of_place(body, types, &out, lent),
                        Rvalue::Cast { kind: CastKind::InterfaceUpcast { concrete, interfaces, implementations, .. }, .. } => {
                            Origins::from([self.table(*concrete, interfaces, implementations)])
                        }
                        Rvalue::Cast { kind: CastKind::ClassInterfaceUpcast { concrete, interface, implementations, .. }, .. } => {
                            Origins::from([self.table(*concrete, &[*interface], implementations)])
                        }
                        Rvalue::Cast { operand, .. } => of_operand(body, types, &out, operand),
                        _ => Origins::from([Origin::Unknown]),
                    };
                    changed |= grow(&mut out, place.local, found);
                }
                let Terminator::Call { func, args, dest, .. } = &block.terminator else { continue };
                if !dest.projection.is_empty() || !out.contains_key(&dest.local) {
                    continue;
                }
                let found = match func {
                    FuncRef::DynBoxNew { concrete, interfaces, implementations, .. } => {
                        Origins::from([self.table(*concrete, interfaces, implementations)])
                    }
                    FuncRef::Builtin { which: Builtin::ClassNew { class_id, .. }, .. } => Origins::from([self.kind(Kind::Class(*class_id))]),
                    FuncRef::Direct { symbol, .. } => match self.index.get(symbol.as_str()).and_then(|&callee| self.returns[callee].as_ref()) {
                        Some(returned) => {
                            let mut found = Origins::new();
                            for origin in returned {
                                match origin {
                                    Origin::Param(param) => match args.get(*param) {
                                        Some(arg) => found.extend(of_operand(body, types, &out, arg)),
                                        None => {
                                            found.insert(Origin::Unknown);
                                        }
                                    },
                                    _ => {
                                        found.insert(*origin);
                                    }
                                }
                            }
                            found
                        }
                        None => Origins::from([Origin::Unknown]),
                    },
                    _ => Origins::from([Origin::Unknown]),
                };
                changed |= grow(&mut out, dest.local, found);
            }
            if !changed {
                return out;
            }
        }
    }

    /// Each body's instances: from the bodies no followed call reaches (every parameter `Unknown`),
    /// what each direct call passes its callee, in each instance of the caller.
    #[allow(clippy::type_complexity)]
    fn instances(&self) -> (Vec<Vec<Vec<Option<Origins>>>>, Vec<Vec<Vec<(usize, usize, usize)>>>) {
        let bodies = self.bodies;
        let mut called = vec![false; bodies.len()];
        for body in bodies {
            for block in &body.blocks {
                if let Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } = &block.terminator
                    && let Some(&callee) = self.index.get(symbol.as_str())
                    && bodies[callee].symbol != body.symbol
                {
                    called[callee] = true;
                }
            }
        }
        let dynamic = crate::stores::dynamic_bodies(bodies);
        let unknown = |at: usize| -> Vec<Option<Origins>> {
            bodies[at].args().map(|(_, local)| carrier(self.types, local.ty).then(|| Origins::from([Origin::Unknown]))).collect()
        };
        let mut instances: Vec<Vec<Vec<Option<Origins>>>> = vec![Vec::new(); bodies.len()];
        // By body and instance, each direct call: its block, its callee, and the callee's instance.
        let mut calls: Vec<Vec<Vec<(usize, usize, usize)>>> = vec![Vec::new(); bodies.len()];
        let mut work: Vec<(usize, usize)> = Vec::new();
        let add = |instances: &mut Vec<Vec<Vec<Option<Origins>>>>,
                   calls: &mut Vec<Vec<Vec<(usize, usize, usize)>>>,
                   work: &mut Vec<(usize, usize)>,
                   at: usize,
                   instance: Vec<Option<Origins>>|
         -> usize {
            let instance = if instances[at].len() >= MOST_INSTANCES && !instances[at].contains(&instance) { unknown(at) } else { instance };
            if let Some(which) = instances[at].iter().position(|seen| *seen == instance) {
                return which;
            }
            instances[at].push(instance);
            calls[at].push(Vec::new());
            work.push((at, instances[at].len() - 1));
            instances[at].len() - 1
        };
        for at in 0..bodies.len() {
            if !called[at] || dynamic.contains(&bodies[at].symbol) {
                add(&mut instances, &mut calls, &mut work, at, unknown(at));
            }
        }
        loop {
            let Some((at, which)) = work.pop() else {
                // A body only calls in a cycle reach (none of them reached): every parameter `Unknown`.
                match (0..bodies.len()).find(|&at| instances[at].is_empty()) {
                    Some(at) => {
                        add(&mut instances, &mut calls, &mut work, at, unknown(at));
                        continue;
                    }
                    None => break,
                }
            };
            let instance = instances[at][which].clone();
            let body = &bodies[at];
            for (block, basic) in body.blocks.iter().enumerate() {
                let Terminator::Call { func: FuncRef::Direct { symbol, .. }, args, .. } = &basic.terminator else { continue };
                let Some(&callee) = self.index.get(symbol.as_str()) else { continue };
                let given: Vec<Option<Origins>> = bodies[callee]
                    .args()
                    .enumerate()
                    .map(|(param, (_, local))| {
                        carrier(self.types, local.ty).then(|| match args.get(param) {
                            Some(arg) => given(&of_operand(body, self.types, &self.origins[at], arg), &instance),
                            None => Origins::from([Origin::Unknown]),
                        })
                    })
                    .collect();
                let given = add(&mut instances, &mut calls, &mut work, callee, given);
                calls[at][which].push((block, callee, given));
            }
        }
        (instances, calls)
    }

    /// The methods the call at `block` of body `at` may run in `instance` (empty: not followed), and
    /// the help where its receiver is a `dyn` parameter whose kind is known only while running.
    fn resolve(&self, at: usize, instance: &[Option<Origins>], block: usize) -> (Vec<Target>, Option<String>) {
        let body = &self.bodies[at];
        let Terminator::Call { func, args, .. } = &body.blocks[block].terminator else { return (Vec::new(), None) };
        let origins = of_operand(body, self.types, &self.origins[at], &args[0]);
        let reach = given(&origins, instance);
        let mut known: Vec<usize> = reach.iter().filter_map(|origin| if let Origin::Kind(kind) = origin { Some(*kind) } else { None }).collect();
        let followed = !reach.contains(&Origin::Unknown);
        let runtime = self.runtime_param(at, instance, block).is_some();
        let mut targets: Vec<Target> = match func {
            FuncRef::Interface { interfaces, slot, .. } if followed || runtime => {
                // `[LT-48]` — a `runtime` parameter's value not followed is any kind the program
                // makes of its table, each checked while running.
                if !followed {
                    known = (0..self.kinds.len())
                        .filter(|&kind| matches!(&self.kinds[kind], Kind::Table { interfaces: held, .. } if held == interfaces))
                        .collect();
                }
                let mut targets = Vec::new();
                for &kind in &known {
                    match self.table_targets(kind, interfaces, *slot) {
                        Some(found) => targets.extend(found),
                        None => return (Vec::new(), self.help(at, &origins, instance, true)),
                    }
                }
                targets
            }
            FuncRef::Virtual { slot, receiver, .. } => {
                let classes: Option<Vec<ClassId>> = known
                    .iter()
                    .map(|&kind| match &self.kinds[kind] {
                        Kind::Class(class) => Some(*class),
                        Kind::Table { .. } => None,
                    })
                    .collect();
                let found: Option<Vec<Target>> = match classes.filter(|_| followed) {
                    Some(classes) => classes.into_iter().map(|class| self.run_by(class, *slot)).collect(),
                    // `[DSP-5]` — the whole program is in the build: any class at or below the call's.
                    None => self
                        .types
                        .runtime_classes()
                        .filter(|(class, def)| def.openness != ClassOpenness::Abstract && self.types.class_is_subclass_of(*class, *receiver))
                        .map(|(class, _)| self.run_by(class, *slot))
                        .collect(),
                };
                // A class whose body in the slot is not found: the worst case.
                found.unwrap_or_default()
            }
            _ => Vec::new(),
        };
        targets.sort();
        targets.dedup();
        let several = targets.iter().map(|target| &target.name).collect::<BTreeSet<_>>().len() > 1;
        // A `runtime` parameter needs no help to write it.
        let help = if runtime { None } else { self.help(at, &origins, instance, targets.is_empty() || several) };
        (targets, help)
    }

    /// `[LT-48]` — the parameter written `runtime` the receiver of the call at `block` comes from,
    /// where its kind in `instance` is known only while running (more than one, or not followed).
    fn runtime_param(&self, at: usize, instance: &[Option<Origins>], block: usize) -> Option<LocalId> {
        let body = &self.bodies[at];
        let Terminator::Call { func: FuncRef::Interface { .. }, args, .. } = &body.blocks[block].terminator else { return None };
        let origins = of_operand(body, self.types, &self.origins[at], args.first()?);
        // The check tests the parameter's value: the receiver must be that value, and nothing else.
        if origins.len() != 1 {
            return None;
        }
        origins.iter().find_map(|origin| {
            let Origin::Param(param) = origin else { return None };
            if !body.runtime_params.contains(param) {
                return None;
            }
            let local = LocalId(*param as u32 + 1);
            let is_dyn = match self.types.kind(body.local(local).ty) {
                TyKind::Ref { inner, .. } => matches!(self.types.kind(*inner), TyKind::Dyn { .. }),
                _ => false,
            };
            let reach = instance.get(*param)?.as_ref()?;
            (is_dyn && (reach.contains(&Origin::Unknown) || reach.len() > 1)).then_some(local)
        })
    }

    /// The flattened layout of the table of `interfaces`, as an upcast to them carries it.
    fn types_layout(&self, interfaces: &[Symbol]) -> Vec<Option<ember_hir::InterfaceSlot>> {
        self.layouts.get(interfaces).cloned().unwrap_or_default()
    }

    /// What a call through the table of `interfaces` at `slot` runs for `kind`: the method the table
    /// holds, and for a class concrete each override of it below the class. `None`: another table.
    fn table_targets(&self, kind: usize, interfaces: &[Symbol], slot: usize) -> Option<Vec<Target>> {
        let Kind::Table { name, interfaces: held, implementations, class, .. } = &self.kinds[kind] else { return None };
        if held.as_slice() != interfaces {
            return None;
        }
        let symbol = implementations.get(slot)?.as_ref()?;
        let mut targets = vec![Target { name: name.clone(), symbol: symbol.clone(), method: self.method_name(symbol), kind: class.is_none().then_some(kind) }];
        if let Some(class) = class
            && let Some(method) = self.index.get(symbol.as_str()).map(|&at| &self.bodies[at])
            && let Some(slot) = method.class_virtual_slot
        {
            for (below, _) in self.types.runtime_classes() {
                if below != *class && self.types.class_is_subclass_of(below, *class) {
                    targets.push(self.run_by(below, slot)?);
                }
            }
        }
        Some(targets)
    }

    /// The body an object made as `class` runs in the virtual `slot`: the nearest at or above it.
    fn run_by(&self, class: ClassId, slot: usize) -> Option<Target> {
        let mut current = Some(class);
        for _ in 0..=self.bodies.len() {
            let at = current?;
            if let Some(&body) = self.defined.get(&(at, slot)) {
                let symbol = self.bodies[body].symbol.clone();
                return Some(Target { name: self.types.class_display_name(class), method: self.method_name(&symbol), symbol, kind: None });
            }
            current = self.types.class_def(at).base;
        }
        None
    }

    /// A method's name, as the program writes it: `pick` of `Mine.pick`.
    fn method_name(&self, symbol: &str) -> String {
        self.index.get(symbol).map_or_else(String::new, |&at| {
            let name = &self.bodies[at].name;
            name.rsplit('.').next().unwrap_or(name).to_string()
        })
    }

    /// `[LT-48]` — the help for a call whose receiver comes from a `dyn` parameter, where `runtime` (the
    /// kind is more than one, or not followed) holds.
    fn help(&self, at: usize, origins: &Origins, instance: &[Option<Origins>], runtime: bool) -> Option<String> {
        let body = &self.bodies[at];
        // `runtime` checks the parameter's value: it helps only where the receiver is that value,
        // and it is not written there already.
        if origins.len() != 1 {
            return None;
        }
        origins.iter().find_map(|origin| {
            let Origin::Param(param) = origin else { return None };
            if body.runtime_params.contains(param) {
                return None;
            }
            let local = body.local(LocalId(*param as u32 + 1));
            let dyn_ty = match self.types.kind(local.ty) {
                TyKind::Dyn { .. } => local.ty,
                TyKind::Ref { inner, .. } if matches!(self.types.kind(*inner), TyKind::Dyn { .. }) => *inner,
                _ => return None,
            };
            let reach = instance.get(*param)?.as_ref()?;
            let several = reach.contains(&Origin::Unknown) || reach.len() > 1;
            if !(runtime && several) {
                return None;
            }
            let function = body.name.rsplit('.').next().unwrap_or(&body.name);
            Some(format!(
                "to check it while running, write `runtime` on the parameter: `fn {function}({}: runtime {}):`",
                local.name.as_deref()?,
                self.types.display(dyn_ty)
            ))
        })
    }
}

/// `origins` in an instance: each parameter replaced by what the instance gives it. Nothing is `Unknown`.
fn given(origins: &Origins, instance: &[Option<Origins>]) -> Origins {
    let mut reach = Origins::new();
    for origin in origins {
        match origin {
            Origin::Param(param) => match instance.get(*param).and_then(Option::as_ref) {
                Some(given) => reach.extend(given.iter().copied()),
                None => {
                    reach.insert(Origin::Unknown);
                }
            },
            _ => {
                reach.insert(*origin);
            }
        }
    }
    if reach.is_empty() {
        reach.insert(Origin::Unknown);
    }
    reach
}

fn grow(out: &mut HashMap<LocalId, Origins>, local: LocalId, found: Origins) -> bool {
    let set = out.entry(local).or_default();
    let before = set.len();
    set.extend(found);
    set.len() != before
}

fn of_operand(body: &Body, types: &TypeTable, out: &HashMap<LocalId, Origins>, operand: &Operand) -> Origins {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => of_place(body, types, out, place),
        Operand::Const(_) => Origins::from([Origin::Unknown]),
    }
}

/// Whether `place` is its local's value itself, read through references and the box holding a `dyn`.
fn transparent(body: &Body, types: &TypeTable, place: &Place) -> bool {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        let through = match (projection, types.kind(ty)) {
            (Projection::Deref, _) => true,
            (Projection::Field(_), TyKind::Struct(id)) => types.compiler_box_inner(*id).is_some(),
            _ => false,
        };
        if !through {
            return false;
        }
        ty = project_type(types, ty, projection, &mut variant);
    }
    true
}

/// What the value at `place` may come from: its local's, read through references and the box holding
/// a `dyn`; any other part of a value (a field, an element) is not followed.
fn of_place(body: &Body, types: &TypeTable, out: &HashMap<LocalId, Origins>, place: &Place) -> Origins {
    let Some(held) = out.get(&place.local) else { return Origins::from([Origin::Unknown]) };
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        let through = match (projection, types.kind(ty)) {
            (Projection::Deref, _) => true,
            (Projection::Field(_), TyKind::Struct(id)) => types.compiler_box_inner(*id).is_some(),
            _ => false,
        };
        if !through {
            return Origins::from([Origin::Unknown]);
        }
        ty = project_type(types, ty, projection, &mut variant);
    }
    held.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_mir::{RegionAccessKind, ResultFieldProvenance, ResultProvenanceSummary};

    fn returning(argument: usize, projection: Vec<Projection>) -> CallRegionContract {
        CallRegionContract {
            access: CallAccessContract::Fields(vec![ParameterFieldAccess { argument, projection: projection.clone(), operations: vec![RegionAccessKind::Read] }]),
            result: CallResultContract::Fields(ResultProvenanceSummary {
                fields: vec![ResultFieldProvenance { result_projection: Vec::new(), sources: vec![ResultRegionSource::View { argument, projection }] }],
            }),
            latebound: false,
            stores: Default::default(),
        }
    }

    fn sources(contract: &CallRegionContract) -> Vec<ResultRegionSource> {
        match &contract.result {
            CallResultContract::Fields(summary) => summary.fields[0].sources.clone(),
            CallResultContract::Legacy(_) => panic!("a joined summary of one shape stays exact"),
        }
    }

    /// `[LT-1]` (verdict 151) — one kind's call is its method's summary, the receiver's paths widened to
    /// the whole receiver; two kinds' is both together: `Mine.pick` returns `this.items`, `Yours.pick`
    /// returns `other`, and a call that may run either points into both.
    #[test]
    fn a_call_that_may_run_several_methods_points_where_each_does() {
        let mine = returning(0, vec![Projection::Deref, Projection::Field(0)]);
        let yours = returning(1, Vec::new());
        let one = join(vec![mine.clone()]);
        assert_eq!(sources(&one), vec![ResultRegionSource::View { argument: 0, projection: Vec::new() }]);
        assert!(one.ties(0) && !one.ties(1));
        let both = join(vec![mine, yours]);
        assert!(both.ties(0) && both.ties(1));
        let CallAccessContract::Fields(fields) = &both.access else { panic!("exact accesses stay exact") };
        assert_eq!(fields.iter().map(|field| (field.argument, field.projection.len())).collect::<Vec<_>>(), vec![(0, 0), (1, 0)]);
        assert_eq!(join(Vec::new()).result, CallResultContract::Legacy(Elision::Everything));
    }

    /// An instance gives each parameter what its caller passed; a place not followed stays `Unknown`.
    #[test]
    fn an_instance_replaces_each_parameter_by_what_the_call_gives_it() {
        let instance = vec![Some(Origins::from([Origin::Kind(3)])), None];
        assert_eq!(given(&Origins::from([Origin::Param(0)]), &instance), Origins::from([Origin::Kind(3)]));
        assert_eq!(given(&Origins::from([Origin::Param(1)]), &instance), Origins::from([Origin::Unknown]));
        assert_eq!(given(&Origins::new(), &instance), Origins::from([Origin::Unknown]));
        assert_eq!(given(&Origins::from([Origin::Param(0), Origin::Kind(1)]), &instance), Origins::from([Origin::Kind(1), Origin::Kind(3)]));
    }
}
