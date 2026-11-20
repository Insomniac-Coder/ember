//! `[ALC-1]`, `[ALC-2]`, `[TYP-46]`, `[OWN-10]` (M10: A1-14, A7-11) — memory managers named with
//! `using`. `T using A` after a type names the kind of manager the value's own memory comes from;
//! `e using inst` after a value names the instance it is made in, which `owns` then holds: `owns
//! Mesh(points=[0.0]) using current` takes the block from `current` by a direct call of its
//! `allocate` and writes the mesh into it. The value is one of std.mem's owners (`OwnsIn` and the
//! three that keep no instance word, verdict 107), shown as `owns T using A` and read as its `T`;
//! its end, std's `drop`, gives the block back through the manager's `deallocate`.

use super::*;

/// What a value made in a manager keeps of its instance (`[ALC-1]`, verdict 107).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManagerKind {
    /// The built-in manager: `owns T using Global` is `owns T`.
    Global,
    /// A manager whose `deallocate` does nothing: nothing is given back, so no word.
    FreesNothing,
    /// A manager with no fields has one instance: no word.
    One,
    /// A manager implementing `FoundByAddress`: the instance is read from the block's piece.
    Found,
    /// Every other: the instance's address, one word.
    Word,
}

const ALLOCATOR: &str = "std.mem.Allocator";

impl Checker<'_> {
    /// `[TYP-46]` — `T using A`: for an `owns T`, the owner whose block comes from an `A`; after a
    /// type with no heap memory of its own, `E2114`; after the other types with memory of their
    /// own, not built yet.
    pub(crate) fn resolve_using_type(&mut self, inner: &ast::TypeExpr, manager: &ast::TypeExpr, span: Span) -> Ty {
        let held = self.resolve_type(inner);
        let kind = self.resolve_type(manager);
        if held == self.common.error || kind == self.common.error {
            return self.common.error;
        }
        // A field's type is resolved before any implementation is known: checked once they are.
        // Before they are, the program's source says whether an `unsafe extend` makes it one.
        if !self.implementations_known {
            if self.source_implementation(kind, "Allocator").is_none() {
                self.pending_managers.push((kind, manager.span));
                return self.common.error;
            }
        } else if !self.check_manager(kind, manager.span) {
            return self.common.error;
        }
        if let ast::TypeKind::Owns(_) = inner.kind
            && let Some(payload) = self.box_inner(held)
            && self.managed_owners.get(&self.struct_of(held)).is_none()
        {
            if matches!(self.types.kind(payload), TyKind::Dyn { .. }) {
                self.error(codes::E0900, span, "`owns dyn I using A` is not built by this compiler yet");
                return self.common.error;
            }
            return self.owns_in(payload, kind);
        }
        if self.has_memory_of_its_own(held) {
            let shown = self.types.display(held);
            let manager = self.types.display(kind);
            self.error(
                codes::E0900,
                span,
                format!("`{shown} using {manager}` is not built by this compiler yet: of the values with memory of their own, only an `owns T` is made in a memory manager so far"),
            );
            return held;
        }
        let written = self.written_line(inner.span);
        self.refuse_no_memory_of_its_own(held, written, kind, span);
        self.common.error
    }

    /// `[TYP-46]` — whether `manager` implements `Allocator`; `E2040` if not.
    fn check_manager(&mut self, manager: Ty, span: Span) -> bool {
        if self.implements(manager, Symbol::intern(ALLOCATOR)) {
            return true;
        }
        let shown = self.types.display(manager);
        self.sink.emit(
            Diagnostic::error(codes::E2040, span, format!("`{shown}` is not a memory manager: it does not implement `std.mem.Allocator`"))
                .note("`using` names the kind of memory manager a value's memory comes from, a type implementing `std.mem.Allocator` [TYP-46]"),
        );
        false
    }

    /// The managers named by types resolved before the implementations were known, checked now.
    pub(crate) fn check_pending_managers(&mut self) {
        for (manager, span) in std::mem::take(&mut self.pending_managers) {
            self.check_manager(manager, span);
        }
    }

    /// `[TYP-46]` — `E2114` ("an `int` has no memory of its own to take from `LevelArena`; remove
    /// `using`").
    fn refuse_no_memory_of_its_own(&mut self, held: Ty, written: Option<String>, manager: Ty, span: Span) {
        let shown = written.unwrap_or_else(|| self.types.display(held));
        let article = if shown.starts_with(['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U']) { "an" } else { "a" };
        let manager = self.types.display(manager);
        self.sink.emit(
            Diagnostic::error(codes::E2114, span, format!("{article} `{shown}` has no memory of its own to take from `{manager}`; remove `using`"))
                .note("`using` names where a value's own heap memory comes from: an `owns`, `shares` or `thread_shared` value's block, a class object, a collection's storage [TYP-46]"),
        );
    }

    /// `[TYP-46]` — whether a value of `ty` has heap memory of its own: an owner's block, a class
    /// object, a link's target, a standard collection whose storage is on the heap.
    fn has_memory_of_its_own(&self, ty: Ty) -> bool {
        match self.types.kind(ty) {
            TyKind::Class(_) | TyKind::ClassInterface(_) | TyKind::Link { .. } | TyKind::Vec { .. } => true,
            TyKind::Struct(id) => {
                self.boxes.contains_key(id)
                    || self.types.struct_def(*id).origin.as_ref().is_some_and(|(name, _)| {
                        matches!(name.as_str(), "std.collections.Map" | "std.collections.Set" | "std.collections.Deque" | "std.collections.SmallArray" | "SoA" | "Shared" | "SyncShared")
                    })
            }
            _ => false,
        }
    }

    fn struct_of(&self, ty: Ty) -> StructId {
        match self.types.kind(ty) {
            TyKind::Struct(id) => *id,
            _ => StructId(u32::MAX),
        }
    }

    /// The manager an `owns T using A` takes its block from; `None` for any other type, an `owns T`
    /// of the built-in manager included.
    pub(crate) fn owner_manager(&self, ty: Ty) -> Option<Ty> {
        self.managed_owners.get(&self.struct_of(ty)).copied()
    }

    /// `[ALC-1]` (verdict 107) — what a value made in `manager` keeps of its instance.
    pub(crate) fn manager_kind(&mut self, manager: Ty) -> ManagerKind {
        if let Some(&kind) = self.manager_kinds.get(&manager) {
            return kind;
        }
        let global = matches!(self.types.kind(manager), TyKind::Struct(id) if self.types.struct_def(*id).name.is("std.mem.Global"));
        let allocator = self.source_implementation(manager, "Allocator");
        let kind = if global {
            ManagerKind::Global
        } else if allocator.is_some_and(|decl| Self::empty_method(&decl, "deallocate")) {
            ManagerKind::FreesNothing
        } else if self.declared_fields(manager) == Some(0) {
            ManagerKind::One
        } else if self.source_implementation(manager, "FoundByAddress").is_some() {
            ManagerKind::Found
        } else {
            ManagerKind::Word
        };
        self.manager_kinds.insert(manager, kind);
        kind
    }

    /// The method `name` by which `manager` implements `Allocator`.
    fn allocator_method(&self, manager: Ty, name: &str) -> Option<DefId> {
        let name = Symbol::intern(name);
        self.interface_methods
            .get(&(manager, Symbol::intern(ALLOCATOR), name))
            .or_else(|| self.methods.get(&(manager, name)))
            .map(|entry| entry.def)
    }

    /// The `unsafe extend` that makes `manager` implement std.mem's `interface` (`Allocator`,
    /// `FoundByAddress`), read from the program's source: what a value made in a manager keeps is
    /// decided where its type is formed, which may be before any implementation is collected.
    fn source_implementation(&mut self, manager: Ty, interface: &str) -> Option<ast::ExtendDecl> {
        let wanted = Symbol::intern(&format!("std.mem.{interface}"));
        let saved = self.current_module;
        let mut found = None;
        for module in 0..self.module_asts.len() {
            let candidates: Vec<ast::ExtendDecl> = self.module_asts[module]
                .items
                .iter()
                .filter_map(|item| match &item.kind {
                    ast::ItemKind::Extend(decl) if decl.generics.is_empty() => Some(decl.clone()),
                    _ => None,
                })
                .collect();
            for decl in candidates {
                self.current_module = module;
                let names = decl.implements.iter().filter_map(interface_name).any(|name| self.resolve_name(name) == wanted);
                if !names {
                    continue;
                }
                let mark = self.sink.mark();
                let target = self.resolve_type(&decl.target);
                self.sink.rollback(mark);
                if target == manager {
                    found = Some(decl);
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        self.current_module = saved;
        found
    }

    /// `[ALC-1]` — "a manager that frees nothing one by one, whose `deallocate` does nothing (an
    /// empty body, checked when compiling)": the extension's method `name` has a body of `pass` alone.
    fn empty_method(decl: &ast::ExtendDecl, name: &str) -> bool {
        decl.members.iter().any(|member| {
            matches!(&member.kind, ast::MemberKind::Fn(method) if method.name.name.is(name)
                && method.body.as_ref().is_some_and(|body| body.stmts.iter().all(|stmt| matches!(stmt.kind, ast::StmtKind::Pass))))
        })
    }

    /// How many fields the struct `manager` declares, read from its declaration when its fields are
    /// not resolved yet; `None` for a type that is not a struct.
    fn declared_fields(&self, manager: Ty) -> Option<usize> {
        let TyKind::Struct(id) = *self.types.kind(manager) else { return None };
        let def = self.types.struct_def(id);
        if !def.fields.is_empty() || def.declaring_module == ember_types::COMPILER_MODULE {
            return Some(def.fields.len());
        }
        let span = def.span;
        for item in self.module_asts.iter().flat_map(|module| module.items.iter()) {
            if let ast::ItemKind::Struct(decl) = &item.kind
                && (decl.name.span == span || item.span == span)
            {
                return Some(decl.members.iter().filter(|member| matches!(member.kind, ast::MemberKind::Field(_))).count());
            }
        }
        Some(0)
    }

    /// `[ALC-1]`, `[TYP-46]` — the type of an `owns T` whose block comes from a `manager`: std.mem's
    /// owner for the manager's kind, shown as `owns T using A` and read through as its `T`.
    pub(crate) fn owns_in(&mut self, inner: Ty, manager: Ty) -> Ty {
        let generic = match self.manager_kind(manager) {
            ManagerKind::Global => return self.box_of(inner),
            ManagerKind::FreesNothing => "std.mem.OwnsFreeNothing",
            ManagerKind::One => "std.mem.OwnsInOne",
            ManagerKind::Found => "std.mem.OwnsFound",
            ManagerKind::Word => "std.mem.OwnsIn",
        };
        let ty = self.instantiate_named_generic(generic, &[inner, manager], Span::DUMMY);
        if let TyKind::Struct(id) = *self.types.kind(ty)
            && !self.managed_owners.contains_key(&id)
        {
            self.boxes.insert(id, inner);
            self.managed_owners.insert(id, manager);
            let shown = format!("owns {} using {}", self.types.display(inner), self.types.display(manager));
            self.types.set_shown_as(id, shown);
        }
        ty
    }

    /// `[GRM-43]` — `e using inst` where no `owns` holds it: only an owner's block is made in an
    /// instance so far; a value with no heap memory of its own is `E2114`.
    pub(crate) fn synth_using(&mut self, value: &ast::Expr, instance: &ast::Expr, expected: Option<Ty>, span: Span) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let place = self.synth_committed(instance);
        let made = self.synth_with_expectation(value, expected);
        if made.ty == self.common.error || place.ty == self.common.error {
            return error;
        }
        let manager = self.peeled(place.ty);
        if self.has_memory_of_its_own(made.ty) {
            let shown = self.types.display(made.ty);
            self.error(
                codes::E0900,
                span,
                format!("a `{shown}` made `using` an instance is not built by this compiler yet: only an `owns T` value is made in a memory manager so far"),
            );
            return error;
        }
        self.refuse_no_memory_of_its_own(made.ty, None, manager, span);
        error
    }

    /// The type a value of `ty` is read as: a view's target.
    fn peeled(&self, ty: Ty) -> Ty {
        match *self.types.kind(ty) {
            TyKind::Ref { inner, .. } => inner,
            _ => ty,
        }
    }

    /// `[ALC-1]`, `[TYP-46]`, `[OWN-10]` — the block of an `owns T` made in a manager: `value` made
    /// `using instance` (or, for a place whose manager has one instance, with none), its block taken
    /// from the instance by a direct call of the manager's `allocate`, a null block stopping the
    /// program ("out of memory: `owns Cmd(id=1) using fixed` (line 8) asked `fixed` for 16 bytes",
    /// `[ALC-6]`), and the value written into it. `expected` is the place's `owns T` type, when known.
    pub(crate) fn owned_block_in(&mut self, value: &ast::Expr, instance: Option<&ast::Expr>, expected: Option<Ty>, span: Span) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let wanted = expected.map(|ty| self.owner_manager(ty));
        // The instance, and the manager it is an instance of.
        let place = match instance {
            Some(instance) => {
                let place = self.synth_committed(instance);
                if place.ty == self.common.error {
                    return error;
                }
                let place = match *self.types.kind(place.ty) {
                    TyKind::Ref { inner, .. } => Expr { ty: inner, kind: ExprKind::Deref(Box::new(place)), span: instance.span },
                    _ => place,
                };
                if !self.implements(place.ty, Symbol::intern(ALLOCATOR)) {
                    let named = self.written_line(instance.span).unwrap_or_else(|| "the instance".to_string());
                    let shown = self.types.display(place.ty);
                    self.error(codes::E2020, instance.span, format!("`{named}` is not an instance of a memory manager: `{shown}` does not implement `std.mem.Allocator`"));
                    return error;
                }
                Some(place)
            }
            None => None,
        };
        let manager = match (&place, wanted.flatten()) {
            (Some(place), _) => place.ty,
            (None, Some(manager)) => manager,
            (None, None) => unreachable!("a block made in a manager names its instance or its place's manager"),
        };
        let kind = self.manager_kind(manager);
        let payload = expected.and_then(|ty| self.box_inner(ty));
        if payload.is_some_and(|payload| matches!(self.types.kind(payload), TyKind::Dyn { .. })) {
            self.error(codes::E0900, span, "`owns dyn I` made in a memory manager is not built by this compiler yet");
            return error;
        }
        let made = match payload {
            Some(payload) => self.check_expr(value, payload),
            None => self.synth_committed(value),
        };
        if made.ty == self.common.error {
            return error;
        }
        // `[TYP-46]` — the place's kind of manager and the instance's agree (`E2020`).
        if let Some(wanted) = wanted {
            let wanted = wanted.unwrap_or(manager);
            let place_kind = wanted;
            let found_global = kind == ManagerKind::Global;
            let wanted_global = self.manager_kind(place_kind) == ManagerKind::Global;
            if place_kind != manager && !(found_global && wanted_global) {
                self.other_manager(made.ty, place_kind, place.as_ref().map(|_| manager), value.span, span);
                return error;
            }
            // Made with no `using`: only a manager with one instance needs none named.
            if place.is_none() && !matches!(kind, ManagerKind::One | ManagerKind::Global) {
                self.other_manager(made.ty, place_kind, None, value.span, span);
                return error;
            }
        }
        if kind == ManagerKind::Global {
            return self.block_around(made, None, span);
        }
        let inner = made.ty;
        let owner = self.owns_in(inner, manager);
        let usize_ty = self.common.usize;
        let placeholder = Expr { ty: inner, kind: ExprKind::Error, span };
        let size = Expr { ty: usize_ty, kind: ExprKind::Builtin { which: Builtin::SizeOf, args: vec![placeholder.clone()] }, span };
        let align = Expr { ty: usize_ty, kind: ExprKind::Builtin { which: Builtin::AlignOf, args: vec![placeholder] }, span };
        let Some(&layout_ty) = self.named_types.get(&Symbol::intern("std.mem.Layout")) else { return error };
        let TyKind::Struct(layout_id) = *self.types.kind(layout_ty) else { return error };
        let layout = Expr { ty: layout_ty, kind: ExprKind::StructLit { struct_id: layout_id, fields: vec![size, align] }, span };
        // A plain parameter of a type that is not `Copy` is passed by address (`[BRW-8]`).
        let layout = if self.types.passed_by_address(layout_ty) { self.borrow_argument(layout, layout_ty) } else { layout };
        let Some(allocate) = self.allocator_method(manager, "allocate") else { return error };
        // The receiver: the instance named, or a manager's one instance (it has no fields).
        let receiver = match &place {
            Some(place) => place.clone(),
            None => {
                let TyKind::Struct(id) = *self.types.kind(manager) else { return error };
                Expr { ty: manager, kind: ExprKind::StructLit { struct_id: id, fields: Vec::new() }, span }
            }
        };
        let mode = self.methods.get(&(manager, Symbol::intern("allocate"))).map_or(Mode::Borrow, |entry| entry.receiver);
        let at = instance.map_or(span, |instance| instance.span);
        // `[ALC-1]` — through a plain parameter, where `allocate` changes the instance, `E3023` says so.
        self.using_receivers.insert(at, (allocate, manager));
        let receiver = self.pass_receiver_to(receiver, mode, allocate, at);
        let byte_ptr = self.types.intern(TyKind::Ptr { mutable: true, inner: self.common.u8 });
        let call = Expr {
            ty: byte_ptr,
            kind: ExprKind::Call { callee: allocate, arg_eval_order: None, default_arg_locals: None, args: vec![receiver, layout], latebound: false },
            span,
        };
        let mut stmts = Vec::new();
        let block = self.hold_value(call, &mut stmts);
        // `[ALC-6]` — the compiler's own call panics on a null block, in the program's words.
        let made_text = self.written_line(span).unwrap_or_else(|| "this value".to_string());
        let line = self.line_of(span).map_or_else(String::new, |line| format!(" (line {line})"));
        let named = instance.and_then(|instance| self.written_line(instance.span)).unwrap_or_else(|| self.types.display(manager));
        let bytes = match self.types.kind(inner) {
            TyKind::Param { .. } => "its block".to_string(),
            _ => {
                let size = self.types.layout(inner).size;
                format!("{} byte{}", with_thousands(size as u128), if size == 1 { "" } else { "s" })
            }
        };
        let message = format!("out of memory: `{made_text}`{line} asked `{named}` for {bytes}");
        let text = Expr { ty: self.common.str_, kind: ExprKind::Str(message), span };
        let panic = Expr { ty: self.common.never, kind: ExprKind::Builtin { which: Builtin::Panic, args: vec![text] }, span };
        let null = Expr { ty: self.common.bool_, kind: ExprKind::Builtin { which: Builtin::PtrIsNull, args: vec![block.clone()] }, span };
        stmts.push(Stmt::If { cond: null, then_block: Block { stmts: vec![Stmt::Expr(panic)], span }, else_block: None });
        let typed = self.types.intern(TyKind::Ptr { mutable: true, inner });
        let address = Expr { ty: typed, kind: ExprKind::Cast { expr: Box::new(block), to: typed }, span };
        let zero = Expr { ty: usize_ty, kind: ExprKind::Int(0), span };
        // The value is handed over into the block: held, then moved, so it ends nowhere else.
        let held = self.hold_value(made, &mut stmts);
        let handed = Expr { ty: held.ty, kind: ExprKind::Builtin { which: Builtin::HandOver, args: vec![held] }, span };
        let write = Expr { ty: self.common.void, kind: ExprKind::Builtin { which: Builtin::PtrWrite, args: vec![address.clone(), zero, handed] }, span };
        stmts.push(Stmt::Expr(write));
        let TyKind::Struct(owner_id) = *self.types.kind(owner) else { return error };
        let mut fields = vec![address];
        if kind == ManagerKind::Word {
            let Some(place) = place else { return error };
            // Only its address: what changes the instance is the calls, each checked where it is made.
            let reference = self.types.intern(TyKind::Ref { mutable: false, inner: manager });
            let reached = Expr { ty: reference, kind: ExprKind::Ref { place: Box::new(place), mutable: false }, span };
            let instance_ptr = self.types.intern(TyKind::Ptr { mutable: true, inner: manager });
            fields.push(Expr { ty: instance_ptr, kind: ExprKind::Cast { expr: Box::new(reached), to: instance_ptr }, span });
        }
        let value = Expr { ty: owner, kind: ExprKind::StructLit { struct_id: owner_id, fields }, span };
        Expr { ty: owner, kind: ExprKind::Block { block: Block { stmts, span }, value: Box::new(value) }, span }
    }

    /// `[TYP-46]` — `E2020` for a value from one manager given to a place of another ("`mesh` takes
    /// its memory from a `LevelArena` (`mesh: owns Mesh using LevelArena`), and this `Mesh` comes from
    /// the built-in manager; say which arena: `mesh=owns Mesh(points=[0.0, 1.0]) using current`").
    /// `found` is the value's manager, `None` for the built-in one.
    fn other_manager(&mut self, inner: Ty, wanted: Ty, found: Option<Ty>, value: Span, span: Span) {
        let holds = |(argument, _): &(Span, Symbol)| argument.file == value.file && argument.start <= value.start && value.end <= argument.end;
        let field = [self.filling, self.argument_of.map(|(argument, name, _)| (argument, name))].into_iter().flatten().filter(holds).min_by_key(|(argument, _)| argument.end - argument.start);
        let place = field.map(|(_, name)| name.to_string()).or_else(|| self.assigned_place(span).map(|(name, _)| name));
        let wanted_shown = self.types.display(wanted);
        let owner_shown = format!("owns {} using {wanted_shown}", self.types.display(inner));
        let noun = Self::common_noun(&wanted_shown);
        let noun = noun.rsplit(' ').next().unwrap_or("manager").to_string();
        let wanted_named = if self.manager_kind(wanted) == ManagerKind::Global { "the built-in manager".to_string() } else { format!("{} `{wanted_shown}`", if wanted_shown.starts_with(['A', 'E', 'I', 'O', 'U']) { "an" } else { "a" }) };
        let comes = match found {
            None => "the built-in manager".to_string(),
            Some(found) => {
                let shown = self.types.display(found);
                format!("{} `{shown}`", if shown.starts_with(['A', 'E', 'I', 'O', 'U']) { "an" } else { "a" })
            }
        };
        let made = self.written_line(span).unwrap_or_else(|| "…".to_string());
        let made = made.split(" using ").next().unwrap_or(&made).to_string();
        let instance = self.instance_in_scope(wanted).unwrap_or_else(|| "…".to_string());
        let shown_inner = self.types.display(inner);
        let message = match (&place, self.manager_kind(wanted) == ManagerKind::Global) {
            (Some(place), false) => format!("`{place}` takes its memory from {wanted_named} (`{place}: {owner_shown}`), and this `{shown_inner}` comes from {comes}; say which {noun}: `{place}={made} using {instance}`"),
            (None, false) => format!("this place takes its memory from {wanted_named} (`{owner_shown}`), and this `{shown_inner}` comes from {comes}; say which {noun}: `{made} using {instance}`"),
            (Some(place), true) => format!("`{place}` takes its memory from the built-in manager (`{place}: owns {shown_inner}`), and this `{shown_inner}` comes from {comes}; make it in the built-in manager: `{place}={made}`"),
            (None, true) => format!("this place takes its memory from the built-in manager (`owns {shown_inner}`), and this `{shown_inner}` comes from {comes}; make it in the built-in manager: `{made}`"),
        };
        self.error(codes::E2020, span, message);
    }

    /// `[ALC-1]` — `E3023` for `using x` through a plain parameter where `allocate` changes the
    /// instance: "`allocate` changes the arena (`LevelArena.allocate`, its line 4), and `current` only
    /// reads it; write `fn fill(ref mut current: LevelArena)`".
    pub(crate) fn using_change_message(&self, write: Span, name: &str, first: &str) -> Option<String> {
        let &(allocate, manager) = self.using_receivers.get(&write)?;
        let shown = self.types.display(manager);
        let noun = Self::common_noun(&shown);
        let noun = noun.rsplit(' ').next().unwrap_or("manager");
        let line = self.line_of(self.signatures[allocate.0 as usize].decl).map_or_else(String::new, |line| format!(", its line {line}"));
        let first = first.strip_suffix(':').unwrap_or(first);
        Some(format!("`allocate` changes the {noun} (`{shown}.allocate`{line}), and `{name}` only reads it; write `{first}`"))
    }

    /// The name of a local in scope that is an instance of `manager` (or a view of one), for a help.
    fn instance_in_scope(&self, manager: Ty) -> Option<String> {
        self.scopes.iter().rev().flat_map(|scope| scope.iter()).find_map(|(name, &local)| {
            let ty = self.locals[local.0 as usize].ty;
            (ty == manager || self.peeled(ty) == manager).then(|| name.to_string())
        })
    }

    /// `[TYP-46]` — a value of one manager's `owns T` given where another's is wanted: `E2020`.
    pub(crate) fn refuse_other_manager(&mut self, expr: &Expr, expected: Ty) -> bool {
        let (found, wanted) = (self.owner_manager(expr.ty), self.owner_manager(expected));
        if found.is_none() && wanted.is_none() {
            return false;
        }
        let (Some(inner), Some(target)) = (self.box_inner(expr.ty), self.box_inner(expected)) else { return false };
        if inner != target {
            return false;
        }
        let wanted = match wanted {
            Some(wanted) => wanted,
            None => match self.named_types.get(&Symbol::intern("std.mem.Global")) {
                Some(&global) => global,
                None => return false,
            },
        };
        self.other_manager(inner, wanted, found, expr.span, expr.span);
        true
    }
}

/// `1073741824` as `1,073,741,824`.
fn with_thousands(n: u128) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn thousands_are_separated() {
        assert_eq!(super::with_thousands(16), "16");
        assert_eq!(super::with_thousands(1_073_741_824), "1,073,741,824");
    }
}
