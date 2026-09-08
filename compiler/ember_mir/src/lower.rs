//! HIR to MIR (Part XVIII §4.5).
//!
//! Expressions are lowered to temporaries; places are preserved; short-circuit
//! operators become branches; every local gets `StorageLive`/`StorageDead`.
//!
//! Phase 0 lowers straight-line code plus `if` and `while`. The drops, retain
//! and release operations, bounds `Assert`s and access brackets that §4.5 also
//! specifies arrive with the phases that give them meaning — Phase 2 for
//! drops, Phase 3 for reference counting.

use ember_hir as hir;
use ember_types::{
    CommonTypes, EnumId, OverflowPolicy, Ty, TyKind, TypeTable, bit_width, is_signed,
};

use crate::{
    AggregateKind, AssertKind, BasicBlock, BasicBlockId, BinOp, Body, CastKind, Const, FuncRef,
    LocalDecl, LocalId, LocalKind, Operand, Place, Projection, RETURN_LOCAL, Rvalue, Stmt,
    StmtKind, Terminator,
};

pub fn lower(program: &hir::Program, types: &TypeTable, common: &CommonTypes) -> Vec<Body> {
    program.functions.iter().map(|f| lower_function(f, program, types, common)).collect()
}

fn lower_function(
    function: &hir::Function,
    program: &hir::Program,
    types: &TypeTable,
    common: &CommonTypes,
) -> Body {
    let mut builder = Builder::new(function, program, types, common);
    builder.build();
    let mut body = builder.finish();
    prune_unreachable(&mut body);
    body
}

/// Drop basic blocks that nothing jumps to and renumber the rest.
///
/// Lowering creates a fresh block after every `return`, `break` and
/// `continue` so that later statements still have somewhere to go. Most of
/// those blocks end up empty and unreachable. Leaving them in would emit
/// unused C labels, and `[CG-C-1]` requires the generated C to compile without
/// warnings — an unused label is one under `-Wall`.
fn prune_unreachable(body: &mut Body) {
    let count = body.blocks.len();
    let mut reachable = vec![false; count];
    let mut stack = vec![0usize];
    while let Some(index) = stack.pop() {
        if reachable[index] {
            continue;
        }
        reachable[index] = true;
        for successor in successors(&body.blocks[index].terminator) {
            stack.push(successor.0 as usize);
        }
    }

    // Old index -> new index, for the blocks that survive.
    let mut remap = vec![None; count];
    let mut next = 0u32;
    for (index, keep) in reachable.iter().enumerate() {
        if *keep {
            remap[index] = Some(BasicBlockId(next));
            next += 1;
        }
    }

    let mut kept = Vec::with_capacity(next as usize);
    for (index, block) in std::mem::take(&mut body.blocks).into_iter().enumerate() {
        if remap[index].is_none() {
            continue;
        }
        let mut block = block;
        retarget(&mut block.terminator, &remap);
        kept.push(block);
    }
    body.blocks = kept;
}

/// Whether a pattern matches every value of its type, so that testing it
/// cannot fail. A binding and a wildcard do; a variant or a literal does not.
fn is_irrefutable(pattern: &hir::Pattern) -> bool {
    match &pattern.kind {
        hir::PatternKind::Wild | hir::PatternKind::Error => true,
        hir::PatternKind::Bind { sub: None, .. } => true,
        hir::PatternKind::Bind { sub: Some(sub), .. } => is_irrefutable(sub),
        hir::PatternKind::Fields(items) => items.iter().all(is_irrefutable),
        hir::PatternKind::Or(alternatives) => alternatives.iter().any(is_irrefutable),
        hir::PatternKind::Variant { .. } | hir::PatternKind::Int(_) => false,
    }
}

fn successors(terminator: &Terminator) -> Vec<BasicBlockId> {
    match terminator {
        Terminator::Goto(bb) => vec![*bb],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut out: Vec<BasicBlockId> = targets.iter().map(|(_, bb)| *bb).collect();
            out.push(*otherwise);
            out
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![*next],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn retarget(terminator: &mut Terminator, remap: &[Option<BasicBlockId>]) {
    let fix = |bb: &mut BasicBlockId| {
        if let Some(new) = remap[bb.0 as usize] {
            *bb = new;
        }
    };
    match terminator {
        Terminator::Goto(bb) => fix(bb),
        Terminator::SwitchInt { targets, otherwise, .. } => {
            for (_, bb) in targets.iter_mut() {
                fix(bb);
            }
            fix(otherwise);
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => fix(next),
        Terminator::Return | Terminator::Unreachable => {}
    }
}

/// One open loop, and where its `break` and `continue` go.
#[derive(Clone, Copy)]
struct Loop {
    continue_bb: BasicBlockId,
    break_bb: BasicBlockId,
    defer_mark: usize,
    owned_mark: usize,
}

struct Builder<'a> {
    function: &'a hir::Function,
    program: &'a hir::Program,
    types: &'a TypeTable,
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    /// Maps a HIR local to the MIR local that holds it.
    local_map: Vec<LocalId>,
    current: BasicBlockId,
    /// Each open loop: where `continue` goes, where `break` goes, and how
    /// many `defer` blocks were pending when it opened — leaving the loop has
    /// to run the ones registered inside it.
    loops: Vec<Loop>,
    /// `[CTL-7]` — `defer` blocks registered in the scopes currently open,
    /// in registration order. They run in reverse.
    defers: Vec<&'a hir::Block>,
    /// `[OWN-2]` — the locals the scopes currently open will drop, in
    /// declaration order. They drop in reverse.
    owned: Vec<LocalId>,
    arg_count: usize,
    /// [TYP-8] -- this function's policy, from its attribute or the profile.
    overflow: OverflowPolicy,
    /// Interned once: every check produces a bool temporary.
    bool_ty: Ty,
    /// Interned once: array indices and lengths are `usize`.
    usize_ty: Ty,
    /// Interned once: the destination of a call that returns nothing.
    void_ty: Ty,
    /// The span statements pushed right now belong to.
    current_span: ember_span::Span,
}

impl<'a> Builder<'a> {
    fn new(
        function: &'a hir::Function,
        program: &'a hir::Program,
        types: &'a TypeTable,
        common: &CommonTypes,
    ) -> Builder<'a> {
        // Local 0 is the return slot; locals 1..=arg_count are the parameters.
        let mut locals = vec![LocalDecl {
            ty: function.ret,
            kind: LocalKind::Return,
            name: None,
            span: function.span,
        }];
        let mut local_map = vec![LocalId(0); function.locals.len()];

        for param in &function.params {
            let decl = function.local(param.local);
            let id = LocalId(locals.len() as u32);
            locals.push(LocalDecl {
                ty: decl.ty,
                kind: LocalKind::Arg,
                name: decl.name.map(|n| n.to_string()),
                span: decl.span,
            });
            local_map[param.local.0 as usize] = id;
        }

        let arg_count = function.params.len();
        for (index, decl) in function.locals.iter().enumerate() {
            if function.params.iter().any(|p| p.local.0 as usize == index) {
                continue;
            }
            let id = LocalId(locals.len() as u32);
            locals.push(LocalDecl {
                ty: decl.ty,
                kind: LocalKind::User,
                name: decl.name.map(|n| n.to_string()),
                span: decl.span,
            });
            local_map[index] = id;
        }

        let blocks = vec![BasicBlock {
            stmts: Vec::new(),
            terminator: Terminator::Unreachable,
            terminator_span: function.span,
        }];
        Builder {
            function,
            program,
            types,
            locals,
            blocks,
            local_map,
            current: BasicBlockId(0),
            loops: Vec::new(),
            defers: Vec::new(),
            owned: Vec::new(),
            arg_count,
            overflow: function.overflow,
            bool_ty: common.bool_,
            usize_ty: common.usize,
            void_ty: common.void,
            current_span: function.span,
        }
    }

    fn finish(self) -> Body {
        Body {
            name: self.function.name.to_string(),
            symbol: self.function.symbol.clone(),
            locals: self.locals,
            blocks: self.blocks,
            arg_count: self.arg_count,
            span: self.function.span,
            borrows: self.function.borrows.clone(),
        }
    }

    // -- block construction --------------------------------------------------

    fn new_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.blocks.len() as u32);
        let span = self.current_span;
        self.blocks.push(BasicBlock {
            stmts: Vec::new(),
            terminator: Terminator::Unreachable,
            terminator_span: span,
        });
        id
    }

    fn push(&mut self, kind: StmtKind) {
        let span = self.current_span;
        self.blocks[self.current.0 as usize].stmts.push(Stmt::new(kind, span));
    }

    /// Record the source location that later `push` calls belong to.
    fn at(&mut self, span: ember_span::Span) {
        if !span.is_dummy() {
            self.current_span = span;
        }
    }

    fn terminate(&mut self, terminator: Terminator) {
        let index = self.current.0 as usize;
        self.blocks[index].terminator_span = self.current_span;
        self.blocks[index].terminator = terminator;
    }

    fn temp(&mut self, ty: Ty, span: ember_span::Span) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span });
        id
    }

    fn build(&mut self) {
        let body = &self.function.body;
        self.lower_block(body);
        // A function whose body falls off the end returns the (void) return
        // slot as it stands. `[FN-8]`'s `main` is the common case.
        if matches!(self.blocks[self.current.0 as usize].terminator, Terminator::Unreachable) {
            self.terminate(Terminator::Return);
        }
    }

    fn lower_block(&mut self, block: &'a hir::Block) {
        let mark = self.defers.len();
        let owned_mark = self.owned.len();
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
        }
        // `[CTL-7]` — leaving the block runs whatever it registered, last
        // first. `[CTL-8]` — the deferred blocks run before the drops.
        self.emit_defers_from(mark);
        self.defers.truncate(mark);
        self.emit_drops_from(owned_mark);
        self.owned.truncate(owned_mark);
    }

    /// `[OWN-2]`, `[DRP-2]` — drop the locals this scope owns, in reverse
    /// declaration order. The list is not shortened: a `return` in one branch
    /// and the end of the block in another both drop the same locals.
    fn emit_drops_from(&mut self, mark: usize) {
        if self.owned.len() <= mark {
            return;
        }
        let pending: Vec<LocalId> = self.owned[mark..].iter().rev().copied().collect();
        for local in pending {
            self.push(StmtKind::Drop { place: Place::local(local), flag: None });
        }
    }

    /// Record a local as this scope's to drop, if its type owns anything.
    fn owns(&mut self, local: LocalId) {
        if self.types.needs_drop(self.locals[local.0 as usize].ty) {
            self.owned.push(local);
        }
    }

    /// Lower every `defer` block registered at or after `mark`, in reverse.
    /// The list is not shortened: a `return` in one branch and the end of the
    /// block in another both have to run the same blocks.
    fn emit_defers_from(&mut self, mark: usize) {
        if self.defers.len() <= mark {
            return;
        }
        let pending: Vec<&'a hir::Block> = self.defers[mark..].iter().rev().copied().collect();
        for block in pending {
            for stmt in &block.stmts {
                self.lower_stmt(stmt);
            }
        }
    }

    fn lower_stmt(&mut self, stmt: &'a hir::Stmt) {
        match stmt {
            hir::Stmt::Let { local, init } => {
                let mir_local = self.local_map[local.0 as usize];
                let decl_span = self.locals[mir_local.0 as usize].span;
                self.at(decl_span);
                self.push(StmtKind::StorageLive(mir_local));
                if let Some(init) = init {
                    let place = Place::local(mir_local);
                    self.lower_into(place, init);
                }
                // `[OWN-1]` — this scope now owns whatever was put here.
                self.owns(mir_local);
            }
            hir::Stmt::Assign { place, value } => {
                let place = self.lower_place(place);
                self.lower_into(place, value);
            }
            hir::Stmt::Expr(expr) => {
                // The value is discarded, but the effects are not.
                let ty = expr.ty;
                let temp = self.temp(ty, expr.span);
                self.lower_into(Place::local(temp), expr);
            }
            hir::Stmt::Return(value) => {
                if let Some(value) = value {
                    self.lower_into(Place::local(RETURN_LOCAL), value);
                }
                // `[CTL-8]` — a `return` runs every `defer` still pending, in
                // every scope it is leaving, and then drops what those scopes
                // own, before it goes.
                self.emit_defers_from(0);
                self.emit_drops_from(0);
                self.terminate(Terminator::Return);
                // Anything after a `return` in the same block is unreachable;
                // start a fresh block so later statements still lower cleanly.
                self.current = self.new_block();
            }
            hir::Stmt::If { cond, then_block, else_block } => {
                let discr = self.lower_operand(cond);
                let then_bb = self.new_block();
                let else_bb = self.new_block();
                let join_bb = self.new_block();
                self.terminate(Terminator::SwitchInt {
                    discr,
                    targets: vec![(0, else_bb)],
                    otherwise: then_bb,
                });

                self.current = then_bb;
                self.lower_block(then_block);
                self.goto_if_open(join_bb);

                self.current = else_bb;
                if let Some(block) = else_block {
                    self.lower_block(block);
                }
                self.goto_if_open(join_bb);

                self.current = join_bb;
            }
            hir::Stmt::While { cond, body, else_block } => {
                let head_bb = self.new_block();
                let body_bb = self.new_block();
                // `[CTL-4]` — falling out of the loop runs `else`; `break`
                // jumps past it. The two exits are separate blocks, so no flag
                // has to be carried at run time.
                let else_bb = self.new_block();
                let exit_bb = self.new_block();
                self.terminate(Terminator::Goto(head_bb));

                self.current = head_bb;
                let discr = self.lower_operand(cond);
                self.terminate(Terminator::SwitchInt {
                    discr,
                    targets: vec![(0, else_bb)],
                    otherwise: body_bb,
                });

                self.loops.push(Loop { continue_bb: head_bb, break_bb: exit_bb, defer_mark: self.defers.len(), owned_mark: self.owned.len() });
                self.current = body_bb;
                self.lower_block(body);
                self.goto_if_open(head_bb);
                self.loops.pop();

                self.current = else_bb;
                if let Some(else_block) = else_block {
                    self.lower_block(else_block);
                }
                self.goto_if_open(exit_bb);

                self.current = exit_bb;
            }
            hir::Stmt::ForRange { local, start, end, inclusive, body, else_block } => {
                self.lower_for_range(*local, start, end, *inclusive, body, else_block.as_ref());
            }
            // `[CTL-7]` — the block runs at scope exit. Registration order is
            // reversed at the end of the enclosing block, so the last one
            // registered runs first.
            hir::Stmt::Defer(block) => self.defers.push(block),
            hir::Stmt::Block(block) => self.lower_block(block),
            hir::Stmt::Break { depth } => {
                if let Some(target) = self.loop_at(*depth).copied() {
                    self.emit_defers_from(target.defer_mark);
                    self.emit_drops_from(target.owned_mark);
                    self.terminate(Terminator::Goto(target.break_bb));
                    self.current = self.new_block();
                }
            }
            hir::Stmt::Continue { depth } => {
                // The continue target is not always the loop head: a counted
                // loop has to run its increment first, or `continue` would
                // spin forever.
                if let Some(target) = self.loop_at(*depth).copied() {
                    self.emit_defers_from(target.defer_mark);
                    self.emit_drops_from(target.owned_mark);
                    self.terminate(Terminator::Goto(target.continue_bb));
                    self.current = self.new_block();
                }
            }
        }
    }

    /// The loop `depth` levels out from the innermost one.
    fn loop_at(&self, depth: usize) -> Option<&Loop> {
        let len = self.loops.len();
        depth.checked_add(1).and_then(|back| len.checked_sub(back)).map(|i| &self.loops[i])
    }

    /// `[CTL-3]` — `for i in a..b`, as a counted loop. The bounds are read
    /// once into locals, and `continue` lands on the increment rather than on
    /// the head, so a `continue` still advances the counter.
    fn lower_for_range(
        &mut self,
        local: hir::LocalId,
        start: &'a hir::Expr,
        end: &'a hir::Expr,
        inclusive: bool,
        body: &'a hir::Block,
        else_block: Option<&'a hir::Block>,
    ) {
        let counter = self.local_map[local.0 as usize];
        let ty = self.function.local(local).ty;

        self.push(StmtKind::StorageLive(counter));
        self.lower_into(Place::local(counter), start);

        // The end is read once, so a loop cannot be changed under itself by
        // its own body.
        let limit = self.temp(ty, end.span);
        self.push(StmtKind::StorageLive(limit));
        self.lower_into(Place::local(limit), end);

        let head_bb = self.new_block();
        let body_bb = self.new_block();
        let step_bb = self.new_block();
        let else_bb = self.new_block();
        let exit_bb = self.new_block();
        self.terminate(Terminator::Goto(head_bb));

        self.current = head_bb;
        let test = self.temp(self.bool_ty, start.span);
        self.push(StmtKind::Assign {
            place: Place::local(test),
            rvalue: Rvalue::BinaryOp {
                op: if inclusive { BinOp::Le } else { BinOp::Lt },
                lhs: Operand::Copy(Place::local(counter)),
                rhs: Operand::Copy(Place::local(limit)),
            },
        });
        self.terminate(Terminator::SwitchInt {
            discr: Operand::Copy(Place::local(test)),
            targets: vec![(0, else_bb)],
            otherwise: body_bb,
        });

        self.loops.push(Loop { continue_bb: step_bb, break_bb: exit_bb, defer_mark: self.defers.len(), owned_mark: self.owned.len() });
        self.current = body_bb;
        self.lower_block(body);
        self.goto_if_open(step_bb);
        self.loops.pop();

        // The counter is stepped with a plain add. `a..=T.MAX` would wrap
        // here; `[CTL-3]` does not say what should happen, and the checked
        // form would cost a branch in every counted loop, so the honest note
        // is that the inclusive form stops short of the type's maximum.
        self.current = step_bb;
        self.push(StmtKind::Assign {
            place: Place::local(counter),
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                lhs: Operand::Copy(Place::local(counter)),
                rhs: Operand::Const(Const::Int { value: 1, ty }),
            },
        });
        self.terminate(Terminator::Goto(head_bb));

        self.current = else_bb;
        if let Some(else_block) = else_block {
            self.lower_block(else_block);
        }
        self.goto_if_open(exit_bb);

        self.current = exit_bb;
    }

    /// Terminate the current block with a jump, unless it already ended.
    fn goto_if_open(&mut self, target: BasicBlockId) {
        if matches!(self.blocks[self.current.0 as usize].terminator, Terminator::Unreachable) {
            self.terminate(Terminator::Goto(target));
        }
    }

    // -- expressions ----------------------------------------------------------

    /// Lower `expr` so that its value ends up in `place`.
    fn lower_into(&mut self, place: Place, expr: &'a hir::Expr) {
        self.at(expr.span);
        match &expr.kind {
            hir::ExprKind::Call { callee, args } => {
                let function = self.program.function(*callee);
                let symbol = function.symbol.clone();
                // `[FN-1]` — the mode decides. `owned` consumes, so the
                // argument is a move and `[OWN-3]`'s analysis sees it;
                // borrowed and `mut` do not, and reading the place for one
                // must not consume it. Threading the mode was outstanding
                // from block A: every argument was borrowed, so an `owned`
                // parameter silently did not take ownership.
                let modes: Vec<hir::Mode> = function.params.iter().map(|p| p.mode).collect();
                let args: Vec<Operand> = args
                    .iter()
                    .enumerate()
                    .map(|(index, a)| match modes.get(index) {
                        Some(hir::Mode::Owned) => self.lower_operand(a),
                        _ => self.lower_operand_borrowed(a),
                    })
                    .collect();
                let next = self.new_block();
                self.terminate(Terminator::Call {
                    func: FuncRef::Direct { symbol },
                    args,
                    dest: place,
                    next,
                });
                self.current = next;
            }
            hir::ExprKind::Builtin { which, args } => {
                // `arg_ty` is what the backend picks its implementation from.
                // For most builtins that is the first argument; `alloc` takes
                // a count and returns the pointer, so its element type is in
                // the result, and `size_of` carries it on a placeholder.
                let arg_ty = match which {
                    hir::Builtin::MemAlloc => expr.ty,
                    hir::Builtin::SizeOf => args.last().map(|a| a.ty).unwrap_or(expr.ty),
                    _ => args.first().map(|a| a.ty).unwrap_or(expr.ty),
                };
                // `push` copies the value through a pointer, so the value has
                // to live somewhere addressable: `&10` is not C.
                let spill = matches!(which, hir::Builtin::ArrayPush);
                let args: Vec<Operand> = args
                    .iter()
                    .enumerate()
                    .map(|(index, a)| {
                        if spill && index == 1 {
                            self.lower_into_temp(a)
                        } else {
                            self.lower_operand_borrowed(a)
                        }
                    })
                    .collect();
                let next = self.new_block();
                self.terminate(Terminator::Call {
                    func: FuncRef::Builtin { which: *which, arg_ty },
                    args,
                    dest: place,
                    next,
                });
                self.current = next;
            }
            // [TYP-8], [TYP-10] -- arithmetic that can trap lowers to the
            // operation plus an Assert, so the check is visible to the effect
            // analysis and the borrow checker rather than hidden in the backend.
            hir::ExprKind::Binary { op, lhs, rhs } if self.needs_check(*op, expr.ty) => {
                self.lower_checked_binary(place, *op, lhs, rhs, expr.ty, expr.span);
            }
            hir::ExprKind::Binary { op, lhs, rhs } if op.is_short_circuit() => {
                // `[EXP-3]` — `and` and `or` short-circuit, so they become
                // branches rather than one `BinaryOp`.
                let lhs_op = self.lower_operand(lhs);
                let rhs_bb = self.new_block();
                let short_bb = self.new_block();
                let join_bb = self.new_block();
                let (zero_target, otherwise) = match op {
                    hir::BinOp::And => (short_bb, rhs_bb),
                    _ => (rhs_bb, short_bb),
                };
                self.terminate(Terminator::SwitchInt {
                    discr: lhs_op,
                    targets: vec![(0, zero_target)],
                    otherwise,
                });

                self.current = rhs_bb;
                let rhs_operand = self.lower_operand(rhs);
                self.push(StmtKind::Assign { place: place.clone(), rvalue: Rvalue::Use(rhs_operand) });
                self.terminate(Terminator::Goto(join_bb));

                self.current = short_bb;
                let short_value = Const::Bool(*op == hir::BinOp::Or);
                self.push(StmtKind::Assign {
                    place: place.clone(),
                    rvalue: Rvalue::Use(Operand::Const(short_value)),
                });
                self.terminate(Terminator::Goto(join_bb));

                self.current = join_bb;
            }
            hir::ExprKind::Match { scrutinee, arms } => {
                self.lower_match(place, scrutinee, arms, expr.span);
            }
            // `[LEX-19]` — start an empty `String` and append each piece.
            hir::ExprKind::FString { parts, buffer_ref } => {
                self.at(expr.span);
                let next = self.new_block();
                self.terminate(Terminator::Call {
                    func: FuncRef::Builtin { which: hir::Builtin::StringNew, arg_ty: expr.ty },
                    args: Vec::new(),
                    dest: place.clone(),
                    next,
                });
                self.current = next;

                let void = self.void_ty;
                for part in parts {
                    // The appenders write through the buffer, so they take its
                    // address — the same shape `s.push_str(...)` produces.
                    let buffer = self.temp(*buffer_ref, expr.span);
                    self.push(StmtKind::StorageLive(buffer));
                    self.push(StmtKind::Assign {
                        place: Place::local(buffer),
                        rvalue: Rvalue::Ref { place: place.clone(), mutable: true },
                    });
                    let target = Operand::Copy(Place::local(buffer));
                    let (which, args, arg_ty) = match part {
                        hir::FStringPart::Text(text) => (
                            hir::Builtin::StringPush,
                            vec![target, Operand::Const(Const::Str(text.clone()))],
                            expr.ty,
                        ),
                        hir::FStringPart::Value(value) => {
                            let operand = self.lower_operand(value);
                            (hir::Builtin::Format, vec![target, operand], value.ty)
                        }
                    };
                    let next = self.new_block();
                    // The append writes through the buffer, not into a
                    // result, so the destination is a throwaway.
                    let sink = self.temp(void, expr.span);
                    self.terminate(Terminator::Call {
                        func: FuncRef::Builtin { which, arg_ty },
                        args,
                        dest: Place::local(sink),
                        next,
                    });
                    self.current = next;
                }
            }
            _ => {
                let rvalue = self.lower_rvalue(expr);
                self.push(StmtKind::Assign { place, rvalue });
            }
        }
    }

    /// `match` (`[ENM-2]`). Each arm tests its pattern, then its guard, then
    /// runs its body; a failed test or guard falls through to the arm below.
    /// The tests themselves are `SwitchInt`s on a discriminant or a value, so
    /// a `match` over an enum becomes a `switch` in the emitted C rather than
    /// a chain of comparisons.
    fn lower_match(
        &mut self,
        dest: Place,
        scrutinee: &'a hir::Expr,
        arms: &'a [hir::MatchArm],
        span: ember_span::Span,
    ) {
        self.at(span);
        // The scrutinee is read once, into a place the patterns project from.
        let scrutinee_place = match &scrutinee.kind {
            hir::ExprKind::Local(_) | hir::ExprKind::Field { .. } => self.lower_place(scrutinee),
            _ => {
                let temp = self.temp(scrutinee.ty, scrutinee.span);
                self.push(StmtKind::StorageLive(temp));
                self.lower_into(Place::local(temp), scrutinee);
                Place::local(temp)
            }
        };

        let join = self.new_block();

        if let Some((id, order)) = self.variant_dispatch(scrutinee.ty, arms) {
            self.lower_variant_dispatch(&dest, &scrutinee_place, (id, &order), arms, join, span);
            self.current = join;
            return;
        }

        let mut arm_entry = self.new_block();
        self.goto_if_open(arm_entry);

        for arm in arms {
            self.current = arm_entry;
            arm_entry = self.new_block();
            self.at(arm.span);
            self.lower_pattern_test(&scrutinee_place, &arm.pattern, arm_entry);

            if let Some(guard) = &arm.guard {
                let passed = self.new_block();
                let discr = self.lower_operand(guard);
                self.terminate(Terminator::SwitchInt {
                    discr,
                    targets: vec![(0, arm_entry)],
                    otherwise: passed,
                });
                self.current = passed;
            }

            match &arm.body {
                hir::MatchArmBody::Block(block) => self.lower_block(block),
                hir::MatchArmBody::Expr(value) => self.lower_into(dest.clone(), value),
            }
            self.goto_if_open(join);
        }

        // Everything refused. `[ENM-2]` makes the arms exhaustive, so this is
        // reachable only in a program that was already reported.
        self.current = arm_entry;
        self.terminate(Terminator::Unreachable);
        self.current = join;
    }

    /// Whether this `match` is the plain shape: one arm per variant, each
    /// naming a different one, no guard, and payloads bound rather than
    /// tested further. That shape needs no test at all beyond reading the tag
    /// once, so it becomes a single `SwitchInt` — a `switch` in the emitted C.
    ///
    /// Anything else falls back to the arm chain, which handles every pattern
    /// form at the cost of one test per arm.
    fn variant_dispatch(&self, ty: Ty, arms: &[hir::MatchArm]) -> Option<(EnumId, Vec<usize>)> {
        let TyKind::Enum(id) = *self.types.kind(ty) else { return None };
        let count = self.types.enum_def(id).variants.len();
        if arms.len() != count || count == 0 {
            return None;
        }
        let mut seen = vec![false; count];
        let mut order = Vec::with_capacity(count);
        for arm in arms {
            if arm.guard.is_some() {
                return None;
            }
            let hir::PatternKind::Variant { variant, fields, .. } = &arm.pattern.kind else {
                return None;
            };
            if seen[*variant] || !fields.iter().all(is_irrefutable) {
                return None;
            }
            seen[*variant] = true;
            order.push(*variant);
        }
        Some((id, order))
    }

    fn lower_variant_dispatch(
        &mut self,
        dest: &Place,
        scrutinee: &Place,
        dispatch: (EnumId, &[usize]),
        arms: &'a [hir::MatchArm],
        join: BasicBlockId,
        span: ember_span::Span,
    ) {
        let (id, order) = dispatch;
        let def = self.types.enum_def(id);
        let repr = def.repr;
        let discriminants: Vec<i128> =
            order.iter().map(|&v| def.variants[v].discriminant).collect();

        let slot = self.temp(repr, span);
        self.push(StmtKind::StorageLive(slot));
        self.push(StmtKind::Assign {
            place: Place::local(slot),
            rvalue: Rvalue::Discriminant(scrutinee.clone()),
        });

        let entries: Vec<BasicBlockId> = arms.iter().map(|_| self.new_block()).collect();
        // Every variant has an arm, so the default is only reached by a tag
        // no variant declares — which the language cannot produce.
        let impossible = self.new_block();
        self.terminate(Terminator::SwitchInt {
            discr: Operand::Copy(Place::local(slot)),
            targets: discriminants.into_iter().zip(entries.iter().copied()).collect(),
            otherwise: impossible,
        });
        self.current = impossible;
        self.terminate(Terminator::Unreachable);

        for ((arm, entry), &variant) in arms.iter().zip(entries).zip(order) {
            self.current = entry;
            self.at(arm.span);
            let hir::PatternKind::Variant { fields, .. } = &arm.pattern.kind else {
                unreachable!("variant_dispatch accepted only variant patterns")
            };
            // The tag is already known, so only the payload bindings remain,
            // and none of them can fail.
            for (index, field) in fields.iter().enumerate() {
                let sub = scrutinee.clone().downcast(variant).field(index);
                self.lower_pattern_test(&sub, field, impossible);
            }
            match &arm.body {
                hir::MatchArmBody::Block(block) => self.lower_block(block),
                hir::MatchArmBody::Expr(value) => self.lower_into(dest.clone(), value),
            }
            self.goto_if_open(join);
        }
    }

    /// Test one pattern against one place, continuing in the current block on
    /// success and jumping to `on_fail` otherwise. Bindings are assigned as
    /// they are passed, which is why a guard sees them.
    fn lower_pattern_test(
        &mut self,
        place: &Place,
        pattern: &hir::Pattern,
        on_fail: BasicBlockId,
    ) {
        match &pattern.kind {
            hir::PatternKind::Wild | hir::PatternKind::Error => {}

            hir::PatternKind::Bind { local, sub } => {
                let target = self.local_map[local.0 as usize];
                let value = self.read(place.clone(), pattern.ty);
                self.push(StmtKind::StorageLive(target));
                self.push(StmtKind::Assign {
                    place: Place::local(target),
                    rvalue: Rvalue::Use(value),
                });
                if let Some(sub) = sub {
                    self.lower_pattern_test(place, sub, on_fail);
                }
            }

            hir::PatternKind::Int(value) => {
                let ok = self.new_block();
                let discr = self.read(place.clone(), pattern.ty);
                self.terminate(Terminator::SwitchInt {
                    discr,
                    targets: vec![(*value, ok)],
                    otherwise: on_fail,
                });
                self.current = ok;
            }

            hir::PatternKind::Variant { enum_id, variant, fields } => {
                let def = self.types.enum_def(*enum_id);
                let tag = def.variants[*variant].discriminant;
                // A one-variant enum needs no test: the tag can only be that.
                if def.variants.len() > 1 {
                    let repr = def.repr;
                    let slot = self.temp(repr, pattern.span);
                    self.push(StmtKind::StorageLive(slot));
                    self.push(StmtKind::Assign {
                        place: Place::local(slot),
                        rvalue: Rvalue::Discriminant(place.clone()),
                    });
                    let ok = self.new_block();
                    self.terminate(Terminator::SwitchInt {
                        discr: Operand::Copy(Place::local(slot)),
                        targets: vec![(tag, ok)],
                        otherwise: on_fail,
                    });
                    self.current = ok;
                }
                for (index, field) in fields.iter().enumerate() {
                    let sub = place.clone().downcast(*variant).field(index);
                    self.lower_pattern_test(&sub, field, on_fail);
                }
            }

            hir::PatternKind::Fields(items) => {
                let is_array = matches!(self.types.kind(pattern.ty), TyKind::Array { .. });
                for (index, item) in items.iter().enumerate() {
                    let sub = if is_array {
                        let mut sub = place.clone();
                        sub.projection.push(Projection::ConstIndex(index as u64));
                        sub
                    } else {
                        place.clone().field(index)
                    };
                    self.lower_pattern_test(&sub, item, on_fail);
                }
            }

            // Each alternative is tried in turn; the first that matches jumps
            // past the rest. Every alternative binds the same locals, which
            // the type checker enforces, so the body sees one set either way.
            hir::PatternKind::Or(alternatives) => {
                let matched = self.new_block();
                let mut attempt = self.current;
                for (index, alternative) in alternatives.iter().enumerate() {
                    self.current = attempt;
                    let last = index + 1 == alternatives.len();
                    attempt = if last { on_fail } else { self.new_block() };
                    self.lower_pattern_test(place, alternative, attempt);
                    self.goto_if_open(matched);
                }
                self.current = matched;
            }
        }
    }

    /// Whether this operator on this type needs a runtime check.
    ///
    /// Division and remainder are always checked: `[TYP-8]` says `/` and `%`
    /// by zero always panic, and `i32.MIN / -1` always panics, with no
    /// dependence on the profile. Overflow of `+ - *` and shift amounts follow
    /// the policy.
    fn needs_check(&self, op: BinOp, ty: Ty) -> bool {
        if !self.types.is_integral(ty) || self.types.is_untyped_literal(ty) {
            return false;
        }
        match op {
            BinOp::Div | BinOp::Rem => true,
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Shl | BinOp::Shr => {
                self.overflow == OverflowPolicy::Panic
            }
            _ => false,
        }
    }

    /// Emit `op` together with the check it needs, leaving the result in
    /// `place` and the cursor on the success path.
    fn lower_checked_binary(
        &mut self,
        place: Place,
        op: BinOp,
        lhs: &'a hir::Expr,
        rhs: &'a hir::Expr,
        ty: Ty,
        span: ember_span::Span,
    ) {
        let lhs_op = self.lower_operand(lhs);
        let rhs_op = self.lower_operand(rhs);

        match op {
            BinOp::Div | BinOp::Rem => {
                // The divisor is zero-checked whatever the policy says.
                let is_zero = self.temp(self.bool_ty, span);
                self.push(StmtKind::Assign {
                    place: Place::local(is_zero),
                    rvalue: Rvalue::BinaryOp {
                        op: BinOp::Eq,
                        lhs: rhs_op.clone(),
                        rhs: Operand::Const(Const::Int { value: 0, ty }),
                    },
                });
                let after_zero = self.new_block();
                self.terminate(Terminator::Assert {
                    cond: Operand::Copy(Place::local(is_zero)),
                    expected: false,
                    msg: AssertKind::DivisionByZero,
                    next: after_zero,
                    span,
                });
                self.current = after_zero;

                // T.MIN / -1 is not representable. The checked helper reports
                // it; unsigned division cannot overflow at all.
                if is_signed(self.types, ty) == Some(true) {
                    let overflow = self.temp(self.bool_ty, span);
                    self.push(StmtKind::CheckedBinaryOp {
                        dest: place,
                        overflow: Place::local(overflow),
                        op,
                        lhs: lhs_op,
                        rhs: rhs_op,
                    });
                    let after = self.new_block();
                    self.terminate(Terminator::Assert {
                        cond: Operand::Copy(Place::local(overflow)),
                        expected: false,
                        msg: AssertKind::SignedDivisionOverflow,
                        next: after,
                        span,
                    });
                    self.current = after;
                } else {
                    self.push(StmtKind::Assign {
                        place,
                        rvalue: Rvalue::BinaryOp { op, lhs: lhs_op, rhs: rhs_op },
                    });
                }
            }

            BinOp::Shl | BinOp::Shr => {
                // [TYP-10] -- a shift amount at or past the width panics under
                // `panic`; `lower_rvalue` masks it under `wrap`.
                let width = bit_width(self.types, ty).unwrap_or(64);
                let too_big = self.temp(self.bool_ty, span);
                self.push(StmtKind::Assign {
                    place: Place::local(too_big),
                    rvalue: Rvalue::BinaryOp {
                        op: BinOp::Ge,
                        lhs: rhs_op.clone(),
                        rhs: Operand::Const(Const::Int { value: width as u128, ty }),
                    },
                });
                let after = self.new_block();
                self.terminate(Terminator::Assert {
                    cond: Operand::Copy(Place::local(too_big)),
                    expected: false,
                    msg: AssertKind::ShiftTooLarge,
                    next: after,
                    span,
                });
                self.current = after;
                self.push(StmtKind::Assign {
                    place,
                    rvalue: Rvalue::BinaryOp { op, lhs: lhs_op, rhs: rhs_op },
                });
            }

            _ => {
                let overflow = self.temp(self.bool_ty, span);
                self.push(StmtKind::CheckedBinaryOp {
                    dest: place,
                    overflow: Place::local(overflow),
                    op,
                    lhs: lhs_op,
                    rhs: rhs_op,
                });
                let after = self.new_block();
                self.terminate(Terminator::Assert {
                    cond: Operand::Copy(Place::local(overflow)),
                    expected: false,
                    msg: AssertKind::Overflow(op),
                    next: after,
                    span,
                });
                self.current = after;
            }
        }
    }

    fn lower_rvalue(&mut self, expr: &'a hir::Expr) -> Rvalue {
        self.at(expr.span);
        match &expr.kind {
            hir::ExprKind::Binary { op: shift @ (BinOp::Shl | BinOp::Shr), lhs, rhs }
                if self.overflow == OverflowPolicy::Wrap
                    && self.types.is_integral(expr.ty)
                    && !self.types.is_untyped_literal(expr.ty) =>
            {
                // [TYP-10] -- under `wrap` the shift amount is masked, which
                // also removes C's undefined behaviour for an over-wide shift.
                let width = bit_width(self.types, expr.ty).unwrap_or(64);
                let lhs = self.lower_operand(lhs);
                let amount = self.lower_operand(rhs);
                let masked = self.temp(expr.ty, expr.span);
                self.push(StmtKind::Assign {
                    place: Place::local(masked),
                    rvalue: Rvalue::BinaryOp {
                        op: BinOp::BitAnd,
                        lhs: amount,
                        rhs: Operand::Const(Const::Int { value: (width - 1) as u128, ty: expr.ty }),
                    },
                });
                Rvalue::BinaryOp { op: *shift, lhs, rhs: Operand::Copy(Place::local(masked)) }
            }
            hir::ExprKind::Binary { op, lhs, rhs } => {
                let lhs = self.lower_operand(lhs);
                let rhs = self.lower_operand(rhs);
                Rvalue::BinaryOp { op: *op, lhs, rhs }
            }
            hir::ExprKind::Unary { op, operand } => {
                let operand = self.lower_operand(operand);
                Rvalue::UnaryOp { op: *op, operand }
            }
            hir::ExprKind::Cast { expr: inner, to } => {
                let operand = self.lower_operand(inner);
                Rvalue::Cast { kind: CastKind::Numeric, operand, to: *to }
            }
            hir::ExprKind::Widen { expr: inner, to } => {
                let operand = self.lower_operand(inner);
                Rvalue::Cast { kind: CastKind::Widen, operand, to: *to }
            }
            hir::ExprKind::StructLit { struct_id, fields } => {
                let operands = fields.iter().map(|f| self.lower_operand(f)).collect();
                Rvalue::Aggregate { kind: AggregateKind::Struct(*struct_id), operands }
            }
            hir::ExprKind::TupleLit(items) => {
                let operands = items.iter().map(|e| self.lower_operand(e)).collect();
                Rvalue::Aggregate { kind: AggregateKind::Tuple, operands }
            }
            hir::ExprKind::ArrayLit(items) => {
                let operands = items.iter().map(|e| self.lower_operand(e)).collect();
                Rvalue::Aggregate { kind: AggregateKind::Array, operands }
            }
            hir::ExprKind::ArrayRepeat { value, count } => {
                let value = self.lower_operand(value);
                Rvalue::Repeat { value, count: *count }
            }
            hir::ExprKind::EnumLit { enum_id, variant, fields } => {
                let operands = fields.iter().map(|f| self.lower_operand(f)).collect();
                Rvalue::Aggregate { kind: AggregateKind::Enum(*enum_id, *variant), operands }
            }
            hir::ExprKind::Ref { place, mutable } => {
                let place = self.lower_place(place);
                Rvalue::Ref { place, mutable: *mutable }
            }
            _ => Rvalue::Use(self.lower_operand(expr)),
        }
    }

    fn lower_operand(&mut self, expr: &'a hir::Expr) -> Operand {
        self.at(expr.span);
        match &expr.kind {
            hir::ExprKind::Int(value) => Operand::Const(Const::Int { value: *value, ty: expr.ty }),
            hir::ExprKind::Float(value) => {
                Operand::Const(Const::Float { value: *value, ty: expr.ty })
            }
            hir::ExprKind::Bool(value) => Operand::Const(Const::Bool(*value)),
            hir::ExprKind::Str(text) => Operand::Const(Const::Str(text.clone())),
            hir::ExprKind::Error => Operand::Const(Const::Void),
            hir::ExprKind::Local(_)
            | hir::ExprKind::Field { .. }
            | hir::ExprKind::Index { .. }
            | hir::ExprKind::Deref(_) => {
                let place = self.lower_place(expr);
                self.read(place, expr.ty)
            }
            // Anything else needs a temporary to hold its value.
            _ => {
                let temp = self.temp(expr.ty, expr.span);
                self.push(StmtKind::StorageLive(temp));
                self.lower_into(Place::local(temp), expr);
                self.read(Place::local(temp), expr.ty)
            }
        }
    }

    /// `a[i]` on a fixed array. Part IV.3 makes the index bounds-checked, so
    /// the comparison and its `Assert` are lowered here rather than left to
    /// the backend — the check is then visible to every MIR analysis, the
    /// same choice `[TYP-8]`'s arithmetic checks make.
    fn lower_index(&mut self, base: &'a hir::Expr, index: &'a hir::Expr, span: ember_span::Span) -> Place {
        let fixed_len = match self.types.kind(base.ty) {
            TyKind::Array { len, .. } => Some(*len),
            // An `Array[T]`'s length is a field, read at the point of use.
            TyKind::Vec { .. } => None,
            // The type checker has already reported this; carry on with a
            // length that makes every access fail rather than pass.
            _ => Some(0),
        };
        let base = self.lower_place(base);
        let len: Operand = match fixed_len {
            Some(len) => Operand::Const(Const::Int { value: len as u128, ty: self.usize_ty }),
            None => Operand::Copy(base.clone().field(1)),
        };

        self.at(span);
        let index_op = self.lower_operand(index);
        let slot = self.temp(self.usize_ty, span);
        self.push(StmtKind::StorageLive(slot));
        self.push(StmtKind::Assign {
            place: Place::local(slot),
            rvalue: Rvalue::Use(index_op),
        });

        let in_range = self.temp(self.bool_ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(in_range),
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Lt,
                lhs: Operand::Copy(Place::local(slot)),
                rhs: len.clone(),
            },
        });
        let after = self.new_block();
        self.terminate(Terminator::Assert {
            cond: Operand::Copy(Place::local(in_range)),
            expected: true,
            msg: AssertKind::Bounds {
                len,
                index: Operand::Copy(Place::local(slot)),
            },
            next: after,
            span,
        });
        self.current = after;

        base.index(slot)
    }

    /// Lower an expression into a fresh local and read it back, so the result
    /// is always something with an address.
    fn lower_into_temp(&mut self, expr: &'a hir::Expr) -> Operand {
        let temp = self.temp(expr.ty, expr.span);
        self.push(StmtKind::StorageLive(temp));
        self.lower_into(Place::local(temp), expr);
        Operand::Copy(Place::local(temp))
    }

    /// An argument read in `[FN-1]`'s default **borrow** mode: the callee sees
    /// the value but does not take it, so the caller still owns it and still
    /// drops it. Only an `owned` parameter consumes — `xs.len()` must not move
    /// `xs` away.
    fn lower_operand_borrowed(&mut self, expr: &'a hir::Expr) -> Operand {
        match self.lower_operand(expr) {
            Operand::Move(place) => Operand::Copy(place),
            other => other,
        }
    }

    /// `[MIR-2]` — a non-`Copy` place is moved, not copied.
    fn read(&self, place: Place, ty: Ty) -> Operand {
        if self.types.is_copy(ty) { Operand::Copy(place) } else { Operand::Move(place) }
    }

    fn lower_place(&mut self, expr: &'a hir::Expr) -> Place {
        match &expr.kind {
            hir::ExprKind::Local(local) => Place::local(self.local_map[local.0 as usize]),
            hir::ExprKind::Field { base, index } => self.lower_place(base).field(*index),
            hir::ExprKind::Index { base, index } => self.lower_index(base, index, expr.span),
            hir::ExprKind::Deref(inner) => {
                let mut place = self.lower_place(inner);
                place.projection.push(Projection::Deref);
                place
            }
            _ => {
                // Not a place expression. The type checker rejects this with
                // `E2140`; MIR gets a temporary so lowering can continue.
                let temp = self.temp(expr.ty, expr.span);
                self.lower_into(Place::local(temp), expr);
                Place::local(temp)
            }
        }
    }
}
