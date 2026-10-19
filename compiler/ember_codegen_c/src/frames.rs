//! `[HR-3b]` (A1, ODR-133) — frame records of a reloadable image with instances. Each function keeps
//! one while it runs (pushed on entry, popped on return): which function it is and which of its calls
//! it is in. For each call, the image lists the package types whose layout the rest of the function
//! depends on once the call returns, so a reload at `std.hot.checkpoint()` applies a layout change
//! the old code still running below the checkpoint no longer uses, and refuses one it does.
//!
//! What the rest of a function depends on: every type whose fields its old code projects (a class's
//! through a handle, a struct's or a tuple's by value), every value it builds, holds, passes or gets
//! back by value (with what that value holds by value), every element type it indexes, every class it
//! allocates, and what the functions it calls directly depend on: those run old code too. A call
//! through a thunk runs the new image's code, which depends on nothing old; a call through a function
//! value may reach any old code, so it depends on what every old body does.

use super::*;

impl Emitter<'_> {
    /// Whether this image keeps frame records: a reloadable build whose instances migrate.
    pub(crate) fn frames_on(&self) -> bool {
        self.unit.is_none() && self.reload.as_ref().is_some_and(|reload| reload.instances)
    }

    /// The package types whose layout a value of `ty` held by value depends on: its own and what it
    /// holds by value. A handle, a pointer and a buffer's elements are behind an address.
    fn by_value_layouts(&self, ty: Ty, out: &mut BTreeSet<String>, seen: &mut Vec<Ty>) {
        if seen.contains(&ty) {
            return;
        }
        seen.push(ty);
        match self.types.kind(ty).clone() {
            TyKind::Struct(id)
                if self.weak_inner_id(id).is_none() && self.shared_inner_id(id).is_none() && self.box_inner_id(id).is_none() =>
            {
                let def = self.types.struct_def(id);
                out.insert(def.name.to_string());
                for field in def.fields.iter().map(|field| field.ty).collect::<Vec<_>>() {
                    self.by_value_layouts(field, out, seen);
                }
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id).clone();
                out.insert(def.name.to_string());
                for field in def.variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)) {
                    self.by_value_layouts(field, out, seen);
                }
            }
            TyKind::Tuple(items) => {
                for item in items {
                    self.by_value_layouts(item, out, seen);
                }
            }
            TyKind::Array { elem, .. } => self.by_value_layouts(elem, out, seen),
            _ => {}
        }
        seen.pop();
    }

    /// What reading or writing `place` depends on: each step's projection into a value's layout.
    fn place_layouts(&self, place: &Place, body: &Body, out: &mut BTreeSet<String>) {
        let mut prefix = Place::local(place.local);
        for step in &place.projection {
            let ty = self.place_ty(&prefix, body);
            match (step, self.types.kind(ty)) {
                (Projection::Field(_) | Projection::Downcast(_), TyKind::Class(id)) => {
                    out.insert(self.types.class_def(*id).name.to_string());
                }
                (Projection::Field(_) | Projection::Downcast(_), _) => self.by_value_layouts(ty, out, &mut Vec::new()),
                (Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_), _) => {
                    let elem = self.element_of(ty);
                    self.by_value_layouts(elem, out, &mut Vec::new());
                }
                (Projection::Deref, _) => {}
            }
            prefix.projection.push(step.clone());
        }
    }

    fn operand_layouts(&self, operand: &Operand, body: &Body, out: &mut BTreeSet<String>) {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            self.place_layouts(place, body, out);
            self.by_value_layouts(self.place_ty(place, body), out, &mut Vec::new());
        }
    }

    /// What one block's code depends on, the bodies it calls directly (old code after a swap), and
    /// whether it calls through a function value.
    fn block_layouts(&self, body: &Body, block: usize, out: &mut BTreeSet<String>, direct: &mut BTreeSet<String>, indirect: &mut bool) {
        for stmt in &body.blocks[block].stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    self.place_layouts(place, body, out);
                    self.by_value_layouts(self.place_ty(place, body), out, &mut Vec::new());
                    match rvalue {
                        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => {
                            self.operand_layouts(operand, body, out);
                        }
                        Rvalue::Repeat { value, .. } => self.operand_layouts(value, body, out),
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            self.operand_layouts(lhs, body, out);
                            self.operand_layouts(rhs, body, out);
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            for operand in operands {
                                self.operand_layouts(operand, body, out);
                            }
                        }
                        Rvalue::Discriminant(place) => {
                            self.place_layouts(place, body, out);
                            self.by_value_layouts(self.place_ty(place, body), out, &mut Vec::new());
                        }
                        Rvalue::Ref { place, .. } => self.place_layouts(place, body, out),
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    for place in [dest, overflow] {
                        self.place_layouts(place, body, out);
                    }
                    self.operand_layouts(lhs, body, out);
                    self.operand_layouts(rhs, body, out);
                }
                // Dropping a value runs its old drop glue; a handle's runs the object's own.
                StmtKind::Drop { place, .. } => {
                    self.place_layouts(place, body, out);
                    self.by_value_layouts(self.place_ty(place, body), out, &mut Vec::new());
                }
                // An access word lives in the object.
                StmtKind::BeginAccess { place, .. }
                | StmtKind::BeginAccessTransfer { place, .. }
                | StmtKind::EndAccess { place, .. }
                | StmtKind::EndAccessTransfer { place, .. } => {
                    self.place_layouts(place, body, out);
                    if let TyKind::Class(id) = self.types.kind(body.locals[place.local.0 as usize].ty) {
                        out.insert(self.types.class_def(*id).name.to_string());
                    }
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            }
        }
        match &body.blocks[block].terminator {
            Terminator::Call { func, args, dest, .. } => {
                for arg in args {
                    self.operand_layouts(arg, body, out);
                }
                self.place_layouts(dest, body, out);
                self.by_value_layouts(self.place_ty(dest, body), out, &mut Vec::new());
                match func {
                    FuncRef::Direct { symbol, .. } if !self.is_reloadable(symbol) => {
                        direct.insert(symbol.clone());
                    }
                    FuncRef::Indirect { operand, .. } => {
                        self.operand_layouts(operand, body, out);
                        *indirect = true;
                    }
                    FuncRef::Builtin { which: Builtin::ClassNew { class_id, .. }, .. } => {
                        out.insert(self.types.class_def(*class_id).name.to_string());
                    }
                    _ => {}
                }
            }
            Terminator::SwitchInt { discr, .. } => self.operand_layouts(discr, body, out),
            Terminator::Assert { cond, .. } => self.operand_layouts(cond, body, out),
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }

    /// What each body of the image depends on, with what the bodies it calls directly depend on;
    /// and what any old body may depend on, for a call through a function value.
    pub(crate) fn plan_frame_layouts(&mut self, bodies: &[Body]) {
        if !self.frames_on() {
            return;
        }
        let mut own: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>, bool)> = BTreeMap::new();
        for body in bodies {
            let (mut uses, mut direct, mut indirect) = (BTreeSet::new(), BTreeSet::new(), false);
            for block in 0..body.blocks.len() {
                self.block_layouts(body, block, &mut uses, &mut direct, &mut indirect);
            }
            own.insert(body.symbol.clone(), (uses, direct, indirect));
        }
        // A direct call reaches only bodies that are not reloadable, so their own uses together are
        // what any old code may depend on.
        let old: BTreeSet<String> = own.iter()
            .filter(|(symbol, _)| !self.is_reloadable(symbol))
            .flat_map(|(_, (uses, _, _))| uses.iter().cloned())
            .collect();
        let mut closed: BTreeMap<String, BTreeSet<String>> = own.iter()
            .map(|(symbol, (uses, _, indirect))| {
                let mut set = uses.clone();
                if *indirect {
                    set.extend(old.iter().cloned());
                }
                (symbol.clone(), set)
            })
            .collect();
        loop {
            let mut changed = false;
            for (symbol, (_, direct, _)) in &own {
                let reached: BTreeSet<String> = direct.iter().filter_map(|callee| closed.get(callee)).flatten().cloned().collect();
                let set = closed.get_mut(symbol).expect("every body has a set");
                for name in reached {
                    changed |= set.insert(name);
                }
            }
            if !changed {
                break;
            }
        }
        self.reload_frame_layouts = closed;
        self.reload_old_layouts = old;
    }

    /// What the rest of `body` depends on once the call ending `block` returns: the call's own
    /// result, and every block reachable from `next`.
    fn continuation_layouts(&self, body: &Body, block: usize, next: usize) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        if let Terminator::Call { dest, .. } = &body.blocks[block].terminator {
            self.place_layouts(dest, body, &mut out);
            self.by_value_layouts(self.place_ty(dest, body), &mut out, &mut Vec::new());
        }
        let mut seen = BTreeSet::new();
        let mut pending = vec![next];
        let (mut direct, mut indirect) = (BTreeSet::new(), false);
        while let Some(at) = pending.pop() {
            if at >= body.blocks.len() || !seen.insert(at) {
                continue;
            }
            self.block_layouts(body, at, &mut out, &mut direct, &mut indirect);
            pending.extend(frame_successors(&body.blocks[at].terminator));
        }
        for callee in &direct {
            if let Some(set) = self.reload_frame_layouts.get(callee) {
                out.extend(set.iter().cloned());
            }
        }
        if indirect {
            out.extend(self.reload_old_layouts.iter().cloned());
        }
        out
    }

    /// `body`'s frame description: its name and, for each call, the source line and the types the
    /// rest of the body depends on. Its call sites are numbered in block order (`rl_sites`).
    pub(crate) fn emit_frame_info(&mut self, body: &Body) -> String {
        let symbol = format!("{}frame_{}", ember_branding::mangle_prefix(), body.symbol);
        self.rl_sites.clear();
        let mut sites = Vec::new();
        for (block, data) in body.blocks.iter().enumerate() {
            let Terminator::Call { next, .. } = &data.terminator else { continue };
            let types = self.continuation_layouts(body, block, next.0 as usize);
            let line = if data.terminator_span.is_dummy() || self.map.get(data.terminator_span.file).is_none() {
                0
            } else {
                self.map.file(data.terminator_span.file).line_col(data.terminator_span.start).line
            };
            self.rl_sites.insert(block, sites.len() as u32);
            sites.push((line, types));
        }
        let mut rows = Vec::new();
        for (index, (line, types)) in sites.iter().enumerate() {
            let list = if types.is_empty() {
                "NULL".to_string()
            } else {
                let names = format!("{symbol}_t{index}");
                let quoted: Vec<String> = types.iter().map(|name| c_string_literal(name)).collect();
                self.line(&format!("static const char* const {names}[{}] = {{ {} }};", types.len(), quoted.join(", ")));
                names
            };
            rows.push(format!("{{ {line}u, {}u, {list} }}", types.len()));
        }
        let sites_ref = if rows.is_empty() {
            "NULL".to_string()
        } else {
            self.line(&format!("static const {RT}rl_site {symbol}_sites[{}] = {{ {} }};", rows.len(), rows.join(", ")));
            format!("{symbol}_sites")
        };
        self.line(&format!(
            "static const {RT}rl_frame_info {symbol} = {{ {}, {}u, {sites_ref} }};",
            c_string_literal(&body.name),
            rows.len()
        ));
        symbol
    }

    /// The site number of the call `terminator` ends, when it is one of `body`'s calls.
    pub(crate) fn frame_site(&self, terminator: &Terminator, body: &Body) -> Option<u32> {
        if !matches!(terminator, Terminator::Call { .. }) {
            return None;
        }
        let block = body.blocks.iter().position(|block| std::ptr::eq(&block.terminator, terminator))?;
        self.rl_sites.get(&block).copied()
    }
}

fn frame_successors(terminator: &Terminator) -> Vec<usize> {
    match terminator {
        Terminator::Goto(next) | Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter().map(|(_, target)| target.0 as usize).chain([otherwise.0 as usize]).collect()
        }
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}
