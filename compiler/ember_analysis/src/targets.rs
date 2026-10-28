//! `[EFF-2]`, `[CLO-17]`, `[EFF-26]` (A4-1) — what a call through a function value or a `dyn` value
//! can run, read from the whole program: one answer for the effect analysis and the summaries.
//!
//! A call through a value of a callable type runs one of the functions the program stores into that
//! exact type: a named function or a lambda that becomes a value of it (a `Const::Fn` stored, passed or
//! built into a value of that type, or the closure of a `Callable` made at it), and whatever a callable
//! type converted to it holds. A call through a `dyn` value runs what one of the program's casts put in
//! that table's slot, or an override of it below its class. A function whose callable type is not known
//! where it becomes a value counts for every callable type of its shape (its parameter count and
//! result): that can only add targets, never lose one.

use std::collections::{HashMap, HashSet};

use ember_mir::{AggregateKind, Body, CastKind, Const, FuncRef, Operand, Rvalue, StmtKind, Terminator};
use ember_span::Symbol;
use ember_types::{ClassId, Ty, TyKind, TypeTable};

use crate::regions::place_type;

/// The targets of the program's calls through values, by body index.
pub struct Targets {
    /// Each callable type, with the bodies the program stores into it.
    values: HashMap<Ty, Vec<usize>>,
    /// The bodies whose callable type is not known where they become values, by parameter count and
    /// result.
    loose: HashMap<(usize, Ty), Vec<usize>>,
    /// Each erased table (its interfaces, in order) at each slot, with what the casts put there and
    /// the overrides of it below its class.
    tables: HashMap<(Vec<Symbol>, usize), Vec<usize>>,
}

impl Targets {
    pub fn new(bodies: &[Body], types: &TypeTable) -> Targets {
        let index: HashMap<&str, usize> = bodies.iter().enumerate().map(|(at, body)| (body.symbol.as_str(), at)).collect();
        let mut found = Found { bodies, types, index: &index, targets: Targets::empty(), conversions: HashSet::new() };
        for body in bodies {
            found.body(body);
        }
        found.finish()
    }

    fn empty() -> Targets {
        Targets { values: HashMap::new(), loose: HashMap::new(), tables: HashMap::new() }
    }

    /// The bodies a call through a value of the callable type `ty` may run. Empty when the program
    /// stores no function into it, which the caller treats as "anything" (a value from elsewhere).
    pub fn of_value(&self, ty: Ty, types: &TypeTable) -> Vec<usize> {
        let mut targets = self.values.get(&ty).cloned().unwrap_or_default();
        if let TyKind::Fn { params, ret, .. } = types.kind(ty) {
            for &loose in self.loose.get(&(params.len(), *ret)).into_iter().flatten() {
                if !targets.contains(&loose) {
                    targets.push(loose);
                }
            }
        }
        targets
    }

    /// The bodies a call through the `dyn` table of `interfaces` at `slot` may run.
    pub fn of_dyn(&self, interfaces: &[Symbol], slot: usize) -> &[usize] {
        self.tables.get(&(interfaces.to_vec(), slot)).map_or(&[], Vec::as_slice)
    }
}

struct Found<'a> {
    bodies: &'a [Body],
    types: &'a TypeTable,
    index: &'a HashMap<&'a str, usize>,
    targets: Targets,
    /// Callable type to callable type: what the first holds, the second may.
    conversions: HashSet<(Ty, Ty)>,
}

impl Found<'_> {
    fn body(&mut self, body: &Body) {
        for block in &body.blocks {
            for stmt in &block.stmts {
                let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
                let dest = place_type(body, self.types, place);
                match rvalue {
                    Rvalue::Use(operand) => {
                        self.converted(body, operand, dest);
                        self.stored(operand, Some(dest));
                    }
                    // `[CLO-3]` — a callable made of a closure and its state, at the callable's type.
                    Rvalue::Aggregate { kind: AggregateKind::Callable, operands } => {
                        if let Some(Operand::Const(Const::Fn(symbol))) = operands.first() {
                            self.add(dest, symbol);
                        }
                        for operand in operands.iter().skip(1) {
                            self.stored(operand, None);
                        }
                    }
                    Rvalue::Aggregate { kind, operands } => {
                        for (at, operand) in operands.iter().enumerate() {
                            let element = self.element(kind, dest, at);
                            self.stored(operand, element);
                        }
                    }
                    Rvalue::Cast { kind, operand, to } => match kind {
                        CastKind::InterfaceUpcast { interfaces, implementations, .. } => self.table(interfaces.clone(), implementations),
                        CastKind::ClassInterfaceUpcast { interface, implementations, .. } => self.table(vec![*interface], implementations),
                        // A function given to C as a pointer is C's to call: `[EFF-26]` counts those
                        // through the callbacks a foreign call is given.
                        CastKind::FnToC => {}
                        _ => {
                            self.converted(body, operand, *to);
                            self.stored(operand, Some(*to));
                        }
                    },
                    Rvalue::UnaryOp { operand, .. } | Rvalue::Repeat { value: operand, .. } => self.stored(operand, None),
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        self.stored(lhs, None);
                        self.stored(rhs, None);
                    }
                    Rvalue::Ref { .. } | Rvalue::Discriminant(_) => {}
                }
            }
            let Terminator::Call { func, args, .. } = &block.terminator else { continue };
            // An argument becomes a value of the parameter's type.
            let params: Vec<Ty> = match func {
                FuncRef::Direct { symbol, .. } => match self.index.get(symbol.as_str()) {
                    Some(&callee) => self.bodies[callee].args().map(|(_, local)| local.ty).collect(),
                    None => Vec::new(),
                },
                FuncRef::Indirect { ty, .. } => match self.types.kind(*ty) {
                    TyKind::Fn { params, .. } => params.iter().map(|param| param.ty).collect(),
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            };
            for (at, operand) in args.iter().enumerate() {
                self.stored(operand, params.get(at).copied());
            }
            match func {
                // A function called through a value it is at once (`(f)(x)` of a named `f`).
                FuncRef::Indirect { operand, ty, .. } => self.stored(operand, Some(*ty)),
                FuncRef::DynBoxNew { interfaces, implementations, .. } => self.table(interfaces.clone(), implementations),
                _ => {}
            }
        }
    }

    /// A value of one callable type stored as another: what the first holds, the second may.
    fn converted(&mut self, body: &Body, operand: &Operand, to: Ty) {
        if let Operand::Copy(source) | Operand::Move(source) = operand {
            let from = place_type(body, self.types, source);
            if from != to && callable(self.types, from) && callable(self.types, to) {
                self.conversions.insert((from, to));
            }
        }
    }

    /// `operand`, which may be a function, becomes a value of `ty` (unknown: every type of its shape).
    fn stored(&mut self, operand: &Operand, ty: Option<Ty>) {
        let Operand::Const(Const::Fn(symbol)) = operand else { return };
        match ty.filter(|&ty| callable(self.types, ty)) {
            Some(ty) => self.add(ty, symbol),
            None => {
                let Some(&target) = self.index.get(symbol.as_str()) else { return };
                let body = &self.bodies[target];
                push(self.targets.loose.entry((body.arg_count, body.return_ty())).or_default(), target);
            }
        }
    }

    fn add(&mut self, ty: Ty, symbol: &str) {
        if let Some(&target) = self.index.get(symbol) {
            push(self.targets.values.entry(ty).or_default(), target);
        }
    }

    /// The type of field `at` of the aggregate `kind` makes, a value of type `dest`.
    fn element(&self, kind: &AggregateKind, dest: Ty, at: usize) -> Option<Ty> {
        match kind {
            AggregateKind::Struct(id) => self.types.struct_def(*id).fields.get(at).map(|field| field.ty),
            AggregateKind::Enum(id, variant) => self.types.enum_def(*id).variants.get(*variant)?.fields.get(at).map(|field| field.ty),
            AggregateKind::Tuple => match self.types.kind(dest) {
                TyKind::Tuple(elements) => elements.get(at).copied(),
                _ => None,
            },
            AggregateKind::Array => match self.types.kind(dest) {
                TyKind::Array { elem, .. } => Some(*elem),
                _ => None,
            },
            AggregateKind::Callable => None,
        }
    }

    fn table(&mut self, interfaces: Vec<Symbol>, implementations: &[Option<ember_mir::InterfaceAdapterMethod>]) {
        for (slot, implementation) in implementations.iter().enumerate() {
            let Some(method) = implementation else { continue };
            let Some(&target) = self.index.get(method.symbol.as_str()) else { continue };
            push(self.targets.tables.entry((interfaces.clone(), slot)).or_default(), target);
        }
    }

    /// What callable types converted into others hold, carried to a fixed point; and below each class
    /// method a table holds, its overrides, which an adapter's virtual call can reach.
    fn finish(mut self) -> Targets {
        let mut changed = true;
        while changed {
            changed = false;
            for &(from, to) in &self.conversions {
                let held = self.targets.values.get(&from).cloned().unwrap_or_default();
                let into = self.targets.values.entry(to).or_default();
                for target in held {
                    changed |= push(into, target);
                }
            }
        }
        let overrides = |target: usize| -> Vec<usize> {
            let method = &self.bodies[target];
            let (Some(slot), Some(owner)) = (method.class_virtual_slot, method.class_owner) else { return Vec::new() };
            self.bodies
                .iter()
                .enumerate()
                .filter(|(_, candidate)| {
                    candidate.class_virtual_slot == Some(slot)
                        && !candidate.is_abstract
                        && candidate.class_owner.is_some_and(|class| below(self.types, class, owner))
                })
                .map(|(at, _)| at)
                .collect()
        };
        let mut tables = std::mem::take(&mut self.targets.tables);
        for held in tables.values_mut() {
            for target in held.clone() {
                for found in overrides(target) {
                    push(held, found);
                }
            }
        }
        self.targets.tables = tables;
        self.targets
    }
}

/// A callable type of Ember's own (a C function pointer declares its effects instead).
fn callable(types: &TypeTable, ty: Ty) -> bool {
    matches!(types.kind(ty), TyKind::Fn { abi: None, .. })
}

/// Whether `class` is `owner` or below it.
fn below(types: &TypeTable, class: ClassId, owner: ClassId) -> bool {
    let mut current = Some(class);
    while let Some(at) = current {
        if at == owner {
            return true;
        }
        current = types.class_def(at).base;
    }
    false
}

/// Add `target` once; whether it was new.
fn push(targets: &mut Vec<usize>, target: usize) -> bool {
    let new = !targets.contains(&target);
    if new {
        targets.push(target);
    }
    new
}
