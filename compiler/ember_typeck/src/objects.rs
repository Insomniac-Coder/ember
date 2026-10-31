//! `[FN-4]`, `[EXC-15]`, `[BRW-10]` — the mode of each method's object that the method does not
//! list (`this`), worked out from the checked bodies before lowering (`M1-plan.md`).
//!
//! Checking types such a method as one that may change its object (`ref mut Self`) and passes its
//! object `ref mut` at each call, holding the read-only checks of that object
//! (`Checker::deferred_objects`). Here, once every body is checked, a method **may change its
//! object** when its body assigns to a place of `this`, takes `ref mut` of one, passes one to a
//! `ref mut` parameter or matches one with a `ref mut` name, or uses `this` otherwise than through a
//! place of it (what it made from it could change the object); or when it calls, on a place of
//! `this`, a method that may: the least fixed point over those calls. The methods of one `virtual`
//! slot take one mode, the union, since one call reaches any of them; so does a call through
//! `dyn`, over the implementations its tables hold. A method that only reads is then made the one
//! `self` was: its object `ref Self` and each call of it a read-only borrow, so lowering, the
//! borrow checks and the back ends see what they saw for a written `self`. The held checks of each
//! call of a method that may change its object are reported.

use super::*;

/// What a call on a place of `this` reaches: a method, or a slot of a `dyn` value's tables.
#[derive(Clone, PartialEq, Eq, Hash)]
enum Reached {
    Method(DefId),
    /// The carrier's bounds, and the slot in their table.
    Slot(Vec<Symbol>, usize),
}

/// Whether `expr` is a place reached from the local `object`: it, or a field, an element or the
/// target of one (`this.items[i].hp`).
fn rooted(expr: &Expr, object: LocalId) -> bool {
    match &expr.kind {
        ExprKind::Local(local) => *local == object,
        ExprKind::Deref(base) | ExprKind::Field { base, .. } | ExprKind::EnumField { base, .. } | ExprKind::Index { base, .. } => {
            rooted(base, object)
        }
        _ => false,
    }
}

/// The `ref mut` a call passes its object as, and the class the method is in when the object is
/// upcast to it (`d.first()` on a `Derived`, a `first` of `Base`).
fn object_borrow(arg: &Expr) -> Option<(&Expr, Option<Ty>)> {
    match &arg.kind {
        ExprKind::Ref { place, mutable: true } => Some((place, None)),
        ExprKind::Cast { expr, to } => match &expr.kind {
            ExprKind::Ref { place, mutable: true } => Some((place, Some(*to))),
            _ => None,
        },
        _ => None,
    }
}

/// Whether a pattern names part of what it matches `ref mut`.
fn binds_mutably(pattern: &hir::Pattern) -> bool {
    match &pattern.kind {
        hir::PatternKind::Bind { by_ref, sub, .. } => *by_ref == Some(true) || sub.as_deref().is_some_and(binds_mutably),
        hir::PatternKind::Variant { fields, .. } | hir::PatternKind::Fields(fields) | hir::PatternKind::Or(fields) => {
            fields.iter().any(binds_mutably)
        }
        hir::PatternKind::Wild | hir::PatternKind::Int(_) | hir::PatternKind::Str(_) | hir::PatternKind::Error => false,
    }
}

/// What one method's body does to its object, as that body alone says.
struct Scan<'s> {
    object: LocalId,
    /// The methods whose calls pass their object as the first argument, `ref mut`.
    implicit: &'s HashSet<DefId>,
    signatures: &'s [Signature],
    types: &'s TypeTable,
    /// The places the program writes that no expression here says it writes (`note_written`).
    written: &'s HashSet<Span>,
    /// The object is a class object's handle: the handle itself, passed on or returned, changes
    /// nothing (any handle may write the object's fields, `[CLS-7]`); a write of a field does.
    handle: bool,
    changes: bool,
    reached: Vec<Reached>,
}

impl Scan<'_> {
    fn block(&mut self, block: &hir::Block) {
        for stmt in &block.stmts {
            self.stmt(stmt);
        }
    }

    fn stmt(&mut self, stmt: &hir::Stmt) {
        match stmt {
            hir::Stmt::Let { init, .. } => init.iter().for_each(|init| self.expr(init)),
            hir::Stmt::Assign { place, value } => {
                self.write(place);
                self.expr(value);
            }
            hir::Stmt::Destructure { value, bindings, .. } => {
                self.expr(value);
                for binding in bindings {
                    match binding {
                        hir::DestructureBinding::Let { value, .. } => self.expr(value),
                        hir::DestructureBinding::Assign { place, value } => {
                            self.write(place);
                            self.expr(value);
                        }
                    }
                }
            }
            hir::Stmt::Expr(expr) => self.expr(expr),
            hir::Stmt::Return(value) => value.iter().for_each(|value| self.expr(value)),
            hir::Stmt::If { cond, then_block, else_block } | hir::Stmt::While { cond, body: then_block, else_block } => {
                self.expr(cond);
                self.block(then_block);
                else_block.iter().for_each(|block| self.block(block));
            }
            hir::Stmt::ForRange { start, end, body, else_block, .. } => {
                self.expr(start);
                self.expr(end);
                self.block(body);
                else_block.iter().for_each(|block| self.block(block));
            }
            hir::Stmt::Block(block) | hir::Stmt::Defer(block) => self.block(block),
            hir::Stmt::Break { .. } | hir::Stmt::Continue { .. } => {}
        }
    }

    /// An assignment's target.
    fn write(&mut self, place: &Expr) {
        self.changes |= rooted(place, self.object);
        self.place(place);
    }

    /// A place: what is read to reach it, its indices.
    fn place(&mut self, place: &Expr) {
        match &place.kind {
            ExprKind::Local(_) => {}
            ExprKind::Deref(base) | ExprKind::Field { base, .. } | ExprKind::EnumField { base, .. } => self.place(base),
            ExprKind::Index { base, index } => {
                self.place(base);
                self.expr(index);
            }
            _ => self.expr(place),
        }
    }

    /// An argument a parameter of `mode` takes.
    fn argument(&mut self, arg: &Expr, mode: Option<Mode>) {
        self.changes |= mode == Some(Mode::Mut) && rooted(arg, self.object);
        self.expr(arg);
    }

    fn expr(&mut self, expr: &Expr) {
        // A place of the object a built-in changes (`this.inner.next()` on a view iterator).
        if self.written.contains(&expr.span) && rooted(expr, self.object) {
            self.changes = true;
        }
        match &expr.kind {
            ExprKind::Local(local) => self.changes |= *local == self.object && !self.handle,
            ExprKind::Deref(_) | ExprKind::Field { .. } | ExprKind::EnumField { .. } | ExprKind::Index { .. }
                if rooted(expr, self.object) =>
            {
                self.place(expr)
            }
            ExprKind::Ref { place, mutable } => {
                self.changes |= *mutable && rooted(place, self.object);
                self.place(place);
            }
            ExprKind::Call { callee, args, .. } => {
                let modes: Vec<Mode> =
                    self.signatures.get(callee.0 as usize).map(|signature| signature.params.iter().map(|param| param.2).collect()).unwrap_or_default();
                for (index, arg) in args.iter().enumerate() {
                    if index == 0
                        && self.implicit.contains(callee)
                        && let Some((place, _)) = object_borrow(arg)
                        && rooted(place, self.object)
                    {
                        self.reached.push(Reached::Method(*callee));
                        self.place(place);
                        continue;
                    }
                    // A class object's handle, the object of a method of it.
                    if index == 0 && self.handle && self.implicit.contains(callee) && rooted(arg, self.object) {
                        self.reached.push(Reached::Method(*callee));
                        self.place(arg);
                        continue;
                    }
                    self.argument(arg, modes.get(index).copied());
                }
            }
            ExprKind::InterfaceCall { interfaces, slot, receiver, args, modes, .. } => {
                // An object a `dyn` box of the object's holds is reached through the box:
                // `Deref(ref mut this.error)`.
                let receiver = match &receiver.kind {
                    ExprKind::Deref(inner) if matches!(&inner.kind, ExprKind::Ref { mutable: true, .. }) => &**inner,
                    _ => &**receiver,
                };
                match &receiver.kind {
                    ExprKind::Ref { place, mutable: true } if rooted(place, self.object) => {
                        self.reached.push(Reached::Slot(interfaces.clone(), *slot));
                        self.place(place);
                    }
                    _ if rooted(receiver, self.object) => {
                        self.reached.push(Reached::Slot(interfaces.clone(), *slot));
                        self.place(receiver);
                    }
                    _ => self.expr(receiver),
                }
                for (index, arg) in args.iter().enumerate() {
                    self.argument(arg, modes.get(index).copied());
                }
            }
            ExprKind::CallIndirect { callee, args, .. } => {
                // A function value whose parameters' words are not known here is taken to change
                // every place it is given.
                let shape = match self.types.kind(callee.ty) {
                    TyKind::FnItem { shape, .. } => *shape,
                    _ => callee.ty,
                };
                let modes: Option<Vec<Mode>> = match self.types.kind(shape) {
                    TyKind::Fn { params, .. } => Some(
                        params
                            .iter()
                            .map(|param| match param.mode {
                                ember_types::FnParamMode::Borrow => Mode::Borrow,
                                ember_types::FnParamMode::Mut => Mode::Mut,
                                ember_types::FnParamMode::Owned => Mode::Owned,
                            })
                            .collect(),
                    ),
                    _ => None,
                };
                self.expr(callee);
                for (index, arg) in args.iter().enumerate() {
                    let mode = modes.as_ref().map_or(Some(Mode::Mut), |modes| modes.get(index).copied());
                    self.argument(arg, mode);
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.changes |= rooted(scrutinee, self.object) && arms.iter().any(|arm| binds_mutably(&arm.pattern));
                self.expr(scrutinee);
                for arm in arms {
                    arm.guard.iter().for_each(|guard| self.expr(guard));
                    match &arm.body {
                        hir::MatchArmBody::Block(block) => self.block(block),
                        hir::MatchArmBody::Expr(body) => self.expr(body),
                    }
                }
            }
            ExprKind::Block { block, value } => {
                self.block(block);
                self.expr(value);
            }
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Str(_)
            | ExprKind::CStr(_)
            | ExprKind::FnValue(_)
            | ExprKind::Error => {}
            ExprKind::Field { base, .. } | ExprKind::EnumField { base, .. } => self.expr(base),
            ExprKind::Builtin { args, .. }
            | ExprKind::StructLit { fields: args, .. }
            | ExprKind::TupleLit(args)
            | ExprKind::ArrayLit(args)
            | ExprKind::EnumLit { fields: args, .. } => args.iter().for_each(|arg| self.expr(arg)),
            ExprKind::InterfaceUpcast { expr: inner, .. }
            | ExprKind::DynBoxNew { value: inner, .. }
            | ExprKind::MakeCallable { env: inner, .. }
            | ExprKind::ArrayRepeat { value: inner, .. }
            | ExprKind::Deref(inner)
            | ExprKind::Unary { operand: inner, .. }
            | ExprKind::Cast { expr: inner, .. }
            | ExprKind::Widen { expr: inner, .. }
            | ExprKind::EraseRange(inner)
            | ExprKind::OverflowScope { expr: inner, .. } => self.expr(inner),
            ExprKind::Index { base, index } => {
                self.expr(base);
                self.expr(index);
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            ExprKind::FString { parts, .. } => parts.iter().filter_map(hir::FStringPart::value).for_each(|value| self.expr(value)),
            ExprKind::ClassNew { default_fields, args, .. } => {
                default_fields.iter().flatten().for_each(|field| self.expr(field));
                args.iter().for_each(|arg| self.expr(arg));
            }
        }
    }
}

impl Checker<'_> {
    /// See the module's comment. `functions` is every checked body, generic instances included.
    /// The methods (by their declaration) the known-numbers rule took to read an object they change
    /// (`Checker::assumed_object_reads`): `check` runs again counting them changes.
    pub(crate) fn decide_implicit_objects(&mut self, functions: &mut [Function]) -> HashSet<Span> {
        let implicit: HashSet<DefId> = (0..self.signatures.len())
            .filter(|&index| self.signatures[index].implicit_object)
            .map(|index| DefId(index as u32))
            .collect();
        if implicit.is_empty() {
            return HashSet::new();
        }
        // The methods whose object is worked out: each whose object is a `ref mut Self` (a
        // `MutSpan`'s is the view itself, and stays it); `init` and `drop` build and end theirs.
        let mut decided: HashMap<DefId, (bool, Vec<Reached>)> = HashMap::new();
        // A generic owner's methods' opaque checks count as their bodies, for what the interface
        // says (`object_reads_by_source`); they are not the program's functions.
        let opaque = std::mem::take(&mut self.opaque_object_bodies);
        for function in functions.iter().chain(opaque.iter().map(|(_, function)| function)) {
            let Some(param) = function.params.first() else { continue };
            if !implicit.contains(&function.def)
                || param.mode != Mode::Mut
                || matches!(function.name.as_str(), "init" | "drop")
                || !matches!(self.types.kind(function.local(param.local).ty), TyKind::Ref { mutable: true, .. } | TyKind::Class(_))
            {
                continue;
            }
            let handle = matches!(self.types.kind(function.local(param.local).ty), TyKind::Class(_));
            let mut scan = Scan { object: param.local, implicit: &implicit, signatures: &self.signatures, types: self.types, written: &self.written_places, handle, changes: false, reached: Vec::new() };
            scan.block(&function.body);
            decided.insert(function.def, (scan.changes, scan.reached));
        }
        // A `virtual` slot's methods, by the class the slot was made in, and each `dyn` table's.
        let root = |types: &TypeTable, mut class: ember_types::ClassId| {
            while let Some(base) = types.class_def(class).base {
                class = base;
            }
            class
        };
        let mut families: HashMap<(ember_types::ClassId, usize), Vec<DefId>> = HashMap::new();
        for function in functions.iter() {
            if let (Some(class), Some(slot)) = (function.class_owner, function.class_virtual_slot) {
                families.entry((root(self.types, class), slot)).or_default().push(function.def);
            }
        }
        let mut tables: HashMap<(Vec<Symbol>, usize), Vec<DefId>> = HashMap::new();
        for function in functions.iter() {
            hir::visit_block(&function.body, &mut |expr| {
                if let ExprKind::InterfaceUpcast { interfaces, implementations, .. } | ExprKind::DynBoxNew { interfaces, implementations, .. } = &expr.kind {
                    for (slot, adapter) in implementations.iter().enumerate() {
                        if let Some(adapter) = adapter {
                            tables.entry((interfaces.clone(), slot)).or_default().push(adapter.implementation);
                        }
                    }
                }
            });
        }
        // A call through a bound names the copy of the method's signature made for that bound
        // (`bound_call_signature`): it is the method's call.
        let origin: HashMap<DefId, DefId> = self.bound_call_signatures.iter().map(|((declaration, ..), &copy)| (copy, *declaration)).collect();
        let origin_of = |def: DefId| origin.get(&def).copied().unwrap_or(def);
        // `[DSP-5]` — an interface's method that does not list its object, called through a bound,
        // is each of the program's implementations of it: it changes its object when one does.
        let mut declared: HashMap<DefId, Vec<DefId>> = HashMap::new();
        for (&(_, interface, method), entry) in &self.interface_methods {
            let declaration = self.interfaces.get(&interface).and_then(|found| {
                found.methods.iter().find(|(name, ..)| *name == method).map(|&(_, declaration, ..)| declaration)
            });
            if let Some(declaration) = declaration.filter(|declaration| implicit.contains(declaration) && *declaration != entry.def) {
                declared.entry(declaration).or_default().push(entry.def);
            }
        }
        // Every such method of an interface, implemented or not: with no implementation in the
        // program (built-in ones aside), no value its call could change exists.
        for found in self.interfaces.values() {
            for &(_, declaration, ..) in &found.methods {
                if implicit.contains(&declaration) {
                    declared.entry(declaration).or_default();
                }
            }
        }
        // The least fixed point. A method not decided here changes its object when it says so,
        // and when its object is implicit and its body is not here (one an interface declares
        // with no implementation in the program).
        let mut changing: HashSet<DefId> = decided.iter().filter(|(_, (changes, _))| *changes).map(|(def, _)| *def).collect();
        let says = |signatures: &[Signature], def: DefId| {
            signatures.get(def.0 as usize).is_none_or(|signature| signature.implicit_object || signature.params.first().is_some_and(|param| param.2 == Mode::Mut))
        };
        let one_changes = |changing: &HashSet<DefId>, def: DefId| {
            if decided.contains_key(&def) { changing.contains(&def) } else { says(&self.signatures, def) }
        };
        // A call through an interface's method: its default body's, if it has one, and each
        // implementation's.
        let changes_object = |changing: &HashSet<DefId>, def: DefId| match declared.get(&origin_of(def)) {
            Some(implementations) => {
                let def = origin_of(def);
                (decided.contains_key(&def) && changing.contains(&def))
                    || self.builtin_objects.get(&def).copied().unwrap_or(false)
                    || implementations.iter().any(|&implementation| one_changes(changing, implementation))
            }
            None => one_changes(changing, origin_of(def)),
        };
        loop {
            let mut grew = false;
            for (def, (_, reached)) in &decided {
                if changing.contains(def) {
                    continue;
                }
                let reaches = reached.iter().any(|reached| match reached {
                    Reached::Method(callee) => changes_object(&changing, *callee),
                    // A `dyn` value of these interfaces is made only where the program converts
                    // to it, every one of which is a table here: with none, there is no such value.
                    Reached::Slot(bounds, slot) => tables
                        .get(&(bounds.clone(), *slot))
                        .is_some_and(|implementations| implementations.iter().any(|&implementation| changes_object(&changing, implementation))),
                });
                if reaches {
                    changing.insert(*def);
                    grew = true;
                }
            }
            for members in families.values() {
                if members.iter().any(|&member| changes_object(&changing, member)) {
                    for member in members {
                        if decided.contains_key(member) && changing.insert(*member) {
                            grew = true;
                        }
                    }
                }
            }
            if !grew {
                break;
            }
        }
        // `[STD-19]` — a method taken to read a relied name's object that changes it.
        let misread: HashSet<Span> = self
            .assumed_object_reads
            .iter()
            .filter(|&&def| changes_object(&changing, def))
            .map(|&def| self.signatures[origin_of(def).0 as usize].decl)
            .collect();
        // Whose own body only reads its object, and through which a call only reads it: the same
        // but for an interface's method, called through a bound, whose implementations count too.
        let reads: HashSet<DefId> = decided.keys().filter(|def| !changing.contains(def)).copied().collect();
        let mut call_reads: HashSet<DefId> = reads.iter().copied().filter(|def| !declared.contains_key(def)).collect();
        call_reads.extend(declared.keys().copied().filter(|&declaration| !changes_object(&changing, declaration)));
        // What the program's interface says of each.
        self.object_reads = call_reads.clone();
        self.object_reads_by_source = opaque.iter().filter(|(_, function)| call_reads.contains(&function.def)).map(|(source, _)| *source).collect();
        // A table's slot: every implementation only reads, those not decided here by what they
        // say (a derive's, which lists its object read-only). With no table, as for the decision
        // above, the program makes no value of these interfaces for a call to change (`[EFF-2]`).
        let slot_reads = |bounds: &Vec<Symbol>, slot: usize| {
            tables.get(&(bounds.clone(), slot)).is_none_or(|implementations| implementations.iter().all(|&implementation| !one_changes(&changing, implementation)))
        };
        // A method that only reads takes its object as a written `self` would (`receiver_by_address`):
        // a read-only view of it, or, a value no view of which its result can be, a copy, its body
        // reading `this` itself. A default body that only reads, whose method another implementation
        // changes, keeps its object `ref mut`: its calls, through the interface, pass it so.
        let mut flipped: HashSet<DefId> = reads.iter().copied().filter(|def| !declared.contains_key(def) || call_reads.contains(def)).collect();
        // `[FFI-39d]` (until M6b removes the run-time checks) — an override of a virtual of a class
        // `import cpp` made, which C++ may call back while it holds the object (a call into C++ keeps
        // its C++ part read): its object is the handle, as the C++ class's own virtual takes it, and
        // its accesses are checked one by one, not as one write of every field for its call. What it
        // does to its object still counts for the rules that read it (`E3023`).
        let cpp_part = Symbol::intern(&format!("{}_cpp_base", ember_branding::SYMBOL_PREFIX));
        let from_cpp: HashSet<DefId> = families
            .iter()
            .filter(|((class, _), _)| self.types.class_def(*class).field(cpp_part).is_some())
            .flat_map(|(_, members)| members.iter().copied())
            .filter(|member| changing.contains(member))
            .collect();
        // A method whose body borrows its object's local itself, as a closure capturing `this` does,
        // keeps the form it was checked with: the closure's environment and body are typed by it.
        // Its calls pass the object as they were checked; what it does to its object still counts.
        // (An override of a C++ class's virtual takes the handle C++ passes, whatever its body.)
        let borrows_object = |function: &Function| {
            let object = function.params[0].local;
            let mut found = false;
            hir::visit_block(&function.body, &mut |expr| {
                if let ExprKind::Ref { place, .. } = &expr.kind
                    && matches!(place.kind, ExprKind::Local(local) if local == object)
                {
                    found = true;
                }
            });
            found
        };
        let kept_form: HashSet<DefId> = functions
            .iter()
            .filter(|function| flipped.contains(&function.def) && !from_cpp.contains(&function.def) && borrows_object(function))
            .map(|function| function.def)
            .collect();
        flipped.retain(|def| !kept_form.contains(def));
        flipped.extend(from_cpp.iter().copied());
        for function in functions.iter_mut().filter(|function| flipped.contains(&function.def)) {
            let object = function.params[0].local;
            function.params[0].mode = Mode::Borrow;
            // A class object's handle stays the handle: only what its call checks changes.
            if matches!(self.types.kind(function.locals[object.0 as usize].ty), TyKind::Class(_)) {
                continue;
            }
            let TyKind::Ref { inner, .. } = *self.types.kind(function.locals[object.0 as usize].ty) else { unreachable!("a decided object is `ref mut`") };
            if self.receiver_by_address(inner, function.def) {
                let view = self.types.intern(TyKind::Ref { mutable: false, inner });
                function.locals[object.0 as usize].ty = view;
                hir::visit_block_mut(&mut function.body, &mut |expr| {
                    if matches!(expr.kind, ExprKind::Local(local) if local == object) {
                        expr.ty = view;
                    }
                });
            } else {
                function.locals[object.0 as usize].ty = inner;
                hir::visit_block_mut(&mut function.body, &mut |expr| match &expr.kind {
                    ExprKind::Deref(base) if matches!(base.kind, ExprKind::Local(local) if local == object) => {
                        *expr = Expr { ty: inner, kind: ExprKind::Local(object), span: expr.span };
                    }
                    ExprKind::Local(local) if *local == object => expr.ty = inner,
                    _ => {}
                });
            }
        }
        // Each call: the object of a method that only reads, as that method takes it, else the held
        // checks reported; each table's adapter calls an implementation as it now takes its object.
        // Whether each such call's callee takes the object by address, asked before the calls change.
        let mut asked: HashSet<(Ty, DefId)> = HashSet::new();
        for function in functions.iter() {
            hir::visit_block(&function.body, &mut |expr| {
                if let ExprKind::Call { callee, args, .. } = &expr.kind
                    && implicit.contains(callee)
                    && let Some((place, upcast)) = args.first().and_then(object_borrow)
                {
                    let taken = upcast.and_then(|to| match *self.types.kind(to) {
                        TyKind::Ref { inner, .. } => Some(inner),
                        _ => None,
                    });
                    asked.insert((taken.unwrap_or(place.ty), *callee));
                }
            });
        }
        let addressed: HashMap<(Ty, DefId), bool> = asked.into_iter().map(|(ty, callee)| ((ty, callee), self.receiver_by_address(ty, callee))).collect();
        let mut held = std::mem::take(&mut self.deferred_objects);
        // Each interface's methods' declarations, which the read-only checks of a call of one are held by.
        let interface_methods: HashMap<Symbol, Vec<Span>> = self
            .interfaces
            .iter()
            .map(|(&name, found)| (name, found.methods.iter().map(|&(_, declaration, ..)| self.signatures[declaration.0 as usize].decl).collect()))
            .collect();
        let mut report = Vec::new();
        // `[CLS-*]` (0.9.10's `@sync`, until M1 refuses it) — a `@sync` class's method may not change
        // its object; one that does not list it is refused once its body is known to.
        for function in functions.iter() {
            if let Some(class) = function.class_owner
                && self.types.class_def(class).is_sync
                && changing.contains(&function.def)
            {
                let owner = self.types.class_def(class).name;
                let at = self.signatures[function.def.0 as usize].decl;
                report.push(
                    Diagnostic::error(ember_diag::codes::E7003, at, format!("`@sync` class `{owner}` cannot declare a method that changes its object: `{}` does", function.name))
                        .help("change its fields through `Atomic` or under a lock, which only read the object"),
                );
            }
        }
        for function in functions.iter_mut() {
            hir::visit_block_mut(&mut function.body, &mut |expr| match &mut expr.kind {
                ExprKind::Call { callee, args, .. } if implicit.contains(callee) => {
                    let Some(object) = args.first_mut() else { return };
                    let reads = call_reads.contains(&origin_of(*callee));
                    if !reads && let Some(diagnostics) = held.remove(&(object.span, self.signatures[callee.0 as usize].decl)) {
                        report.extend(diagnostics);
                    }
                    if (reads || from_cpp.contains(&origin_of(*callee))) && !kept_form.contains(&origin_of(*callee)) {
                        // As the callee takes it: a read-only view, or the value; under the upcast
                        // to the class the method is in, when there is one.
                        if let Some((place, upcast)) = object_borrow(object) {
                            let place = place.clone();
                            let taken = upcast.and_then(|to| match *self.types.kind(to) {
                                TyKind::Ref { inner, .. } => Some(inner),
                                _ => None,
                            });
                            let inner = taken.unwrap_or(place.ty);
                            let span = object.span;
                            let value = if addressed.get(&(inner, *callee)).copied().unwrap_or(true) {
                                let view = self.types.intern(TyKind::Ref { mutable: false, inner: place.ty });
                                let borrowed = Expr { ty: view, kind: ExprKind::Ref { place: Box::new(place), mutable: false }, span };
                                match taken {
                                    Some(_) => {
                                        let to = self.types.intern(TyKind::Ref { mutable: false, inner });
                                        Expr { ty: to, kind: ExprKind::Cast { expr: Box::new(borrowed), to }, span }
                                    }
                                    None => borrowed,
                                }
                            } else {
                                match taken {
                                    Some(_) => Expr { ty: inner, kind: ExprKind::Cast { expr: Box::new(place), to: inner }, span },
                                    None => place,
                                }
                            };
                            *object = value;
                        }
                    }
                }
                ExprKind::InterfaceCall { interfaces, interface, slot, receiver, .. } => {
                    if slot_reads(interfaces, *slot) {
                        if let ExprKind::Ref { mutable, place } = &mut receiver.kind
                            && *mutable
                        {
                            *mutable = false;
                            let inner = place.ty;
                            receiver.ty = self.types.intern(TyKind::Ref { mutable: false, inner });
                        }
                    } else {
                        // Held as a `dyn` value's call, or, a class object reached as its interface
                        // (`[OBJ-2]`), as a call of the interface's method.
                        let declared = interface_methods.get(interface).map(Vec::as_slice).unwrap_or_default();
                        for decl in std::iter::once(&Span::DUMMY).chain(declared) {
                            if let Some(diagnostics) = held.remove(&(receiver.span, *decl)) {
                                report.extend(diagnostics);
                            }
                        }
                    }
                }
                ExprKind::InterfaceUpcast { implementations, .. } | ExprKind::DynBoxNew { implementations, .. } => {
                    for adapter in implementations.iter_mut().flatten() {
                        if flipped.contains(&adapter.implementation) {
                            adapter.receiver = Mode::Borrow;
                        }
                    }
                }
                _ => {}
            });
        }
        self.sink.emit_all(report);
        misread
    }
}
