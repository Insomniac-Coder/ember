//! `[OWN-2]`, `[DRP-6]`, `[DRP-8]`, `[RC-4]` — what ending a value runs: each `drop` it may reach (its own
//! type's, its parts', a class's and those below it, a lambda's captured state's, what a `Box[dyn I]` or an
//! interface handle was made from, the object a counted link may be the last owner of) and what the end
//! itself is (a scope's join, a guard's release). One walk, which the effects and the summaries both read.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use ember_mir::{Body, CastKind, FuncRef, Rvalue, StmtKind, Terminator};
use ember_types::{StructId, Ty, TyKind, TypeTable};

use crate::targets::Targets;

/// What ending a value of one type runs.
#[derive(Default, Debug)]
pub(crate) struct DropRun {
    /// Each `drop` body it may run, and whether that is the type's own (its `this` is the value that
    /// ends) rather than a part's or something the value owns.
    pub(crate) bodies: Vec<(usize, bool)>,
    /// `[THR-3]`, `[THR-5]`, `[JOB-2]` — a lock released, a scope's threads joined or its jobs waited
    /// for, a `Sync` object's handle released: a synchronising operation.
    pub(crate) sync: bool,
    /// A scope's end waits for its threads or jobs.
    pub(crate) block: bool,
    /// Something whose end cannot be followed here (a C function value's state).
    pub(crate) unknown: bool,
}

/// The walk over one program's bodies, each type's result kept (the borrow check reads it from its
/// threads).
pub(crate) struct DropRuns<'a> {
    bodies: &'a [Body],
    types: &'a TypeTable,
    index: &'a HashMap<&'a str, usize>,
    /// Each erased type (`dyn I` in a box, an interface handle), with the types the program makes into it.
    erased: HashMap<Ty, Vec<Ty>>,
    known: Mutex<HashMap<Ty, Arc<DropRun>>>,
}

impl<'a> DropRuns<'a> {
    pub(crate) fn new(bodies: &'a [Body], types: &'a TypeTable, index: &'a HashMap<&'a str, usize>) -> Self {
        let mut erased: HashMap<Ty, Vec<Ty>> = HashMap::new();
        for body in bodies {
            for block in &body.blocks {
                for stmt in &block.stmts {
                    if let StmtKind::Assign { rvalue: Rvalue::Cast { kind: CastKind::ClassInterfaceUpcast { concrete, .. }, to, .. }, .. } = &stmt.kind {
                        erased.entry(*to).or_default().push(*concrete);
                    }
                }
                if let Terminator::Call { func: FuncRef::DynBoxNew { concrete, interfaces, .. }, .. } = &block.terminator
                    && let Some(erased_ty) = types.find(&TyKind::Dyn { interfaces: interfaces.clone() })
                {
                    erased.entry(erased_ty).or_default().push(*concrete);
                }
            }
        }
        for made in erased.values_mut() {
            made.sort_unstable();
            made.dedup();
        }
        DropRuns { bodies, types, index, erased, known: Mutex::new(HashMap::new()) }
    }

    /// What ending a value of `ty` runs; `targets` says which functions a function value may be.
    pub(crate) fn of(&self, ty: Ty, targets: &Targets) -> Arc<DropRun> {
        if let Some(known) = self.known.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(&ty) {
            return known.clone();
        }
        let mut run = DropRun::default();
        self.walk(ty, true, targets, &mut HashSet::new(), &mut run);
        run.bodies.sort_unstable();
        run.bodies.dedup();
        let run = Arc::new(run);
        self.known.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(ty, run.clone());
        run
    }

    fn walk(&self, ty: Ty, own: bool, targets: &Targets, seen: &mut HashSet<Ty>, run: &mut DropRun) {
        if !seen.insert(ty) {
            return;
        }
        let types = self.types;
        let mut parts: Vec<Ty> = Vec::new();
        let drop_body = |symbol: String, run: &mut DropRun| {
            if let Some(&body) = self.index.get(symbol.as_str()) {
                run.bodies.push((body, own));
            }
        };
        match types.kind(ty).clone() {
            // `[THR-5]`, `[JOB-2]` — a `Scope`'s end joins its threads, a `JobScope`'s waits for its jobs.
            TyKind::Struct(id) if origin_is(types, id, &["std.thread.Scope", "std.jobs.JobScope"]) => {
                run.sync = true;
                run.block = true;
            }
            // `[THR-3]` — a guard's end releases its lock.
            TyKind::Struct(id) if origin_is(types, id, &["std.sync.MutexGuard", "std.sync.RwLockReadGuard", "std.sync.RwLockWriteGuard", "std.thread.JoinHandle"]) => {
                run.sync = true;
            }
            TyKind::Struct(id) => {
                let def = types.struct_def(id);
                if def.has_drop {
                    drop_body(types.method_symbol(ty, None, "drop", None), run);
                }
                parts.extend(def.fields.iter().map(|field| field.ty));
                if let Some((_, args)) = &def.origin {
                    parts.extend(args.iter().copied());
                }
            }
            TyKind::Enum(id) => {
                let def = types.enum_def(id);
                if def.has_drop {
                    drop_body(types.method_symbol(ty, None, "drop", None), run);
                }
                parts.extend(def.variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)));
            }
            TyKind::Tuple(items) => parts.extend(items),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } | TyKind::Vec { elem, .. } => parts.push(elem),
            // The last handle's release ends the object, whose class may be any below `id`.
            TyKind::Class(id) => {
                for (class, _) in types.runtime_classes() {
                    let mut chain = Vec::new();
                    let mut current = Some(class);
                    while let Some(link) = current {
                        chain.push(link);
                        current = types.class_def(link).base;
                    }
                    if !chain.contains(&id) {
                        continue;
                    }
                    if types.class_def(class).is_sync {
                        run.sync = true;
                    }
                    for link in chain {
                        let def = types.class_def(link);
                        if def.has_drop {
                            drop_body(types.class_method_symbol(link, "drop"), run);
                        }
                        parts.extend(def.fields.iter().map(|field| field.ty));
                    }
                }
            }
            // `[RC-4]`, `[WK-11]` — an owner may be the last one; a link keeps nothing alive.
            TyKind::Link { kind, target } if !kind.is_weak() => parts.push(target),
            // What the program made into the erased type.
            TyKind::Dyn { .. } | TyKind::ClassInterface(_) => parts.extend(self.erased.get(&ty).into_iter().flatten().copied()),
            // `[EFF-2]` (ODR 15.3) — a function value's end is that of one of the functions stored into its
            // type: none for a named function, its captured values' for a lambda.
            TyKind::Fn { stateless: false, abi: None, .. } => {
                for target in targets.of_value(ty, types) {
                    if let Some(environment) = self.bodies[target].closure_environment {
                        match types.find(&TyKind::Struct(environment)) {
                            Some(captured) => parts.push(captured),
                            None => run.unknown = true,
                        }
                    }
                }
            }
            TyKind::Fn { stateless: false, .. } => run.unknown = true,
            _ => {}
        }
        for part in parts {
            if types.needs_drop(part) {
                self.walk(part, false, targets, seen, run);
            }
        }
    }
}

/// Whether struct `id` is an instance of one of `origins` (`std.sync.MutexGuard`, ...).
fn origin_is(types: &TypeTable, id: StructId, origins: &[&str]) -> bool {
    types.struct_def(id).origin.as_ref().is_some_and(|(origin, _)| origins.contains(&origin.as_str()))
}
