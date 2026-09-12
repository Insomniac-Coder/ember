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
use ember_span::SourceMap;
use ember_types::{
    CommonTypes, EnumId, OverflowPolicy, Ty, TyKind, TypeTable, bit_width, is_signed,
};

use crate::{
    AggregateKind, AssertKind, BasicBlock, BasicBlockId, BinOp, Body, CastKind, Const, FuncRef,
    LocalDecl, LocalId, LocalKind, Operand, Place, Projection, RETURN_LOCAL, Rvalue, Stmt,
    StmtKind, Terminator,
};

pub fn lower(
    program: &hir::Program,
    types: &TypeTable,
    common: &CommonTypes,
    map: &SourceMap,
) -> Vec<Body> {
    program.functions.iter().map(|f| lower_function(f, program, types, common, map)).collect()
}

fn lower_function(
    function: &hir::Function,
    program: &hir::Program,
    types: &TypeTable,
    common: &CommonTypes,
    map: &SourceMap,
) -> Body {
    let mut builder = Builder::new(function, program, types, common, map);
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
    /// `[CELL-5]` — source locations for the conflicting-borrow panic.
    /// Lowering needs file paths and line numbers, which live in the map.
    map: &'a SourceMap,
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
    /// `[DRP-3]` — the temporaries the statement being lowered has made, and
    /// which end with it. Named locals go in `owned` and end with their block.
    statement_temps: Vec<LocalId>,
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
        map: &'a SourceMap,
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
        // `[FN-1]` transfers an `owned` argument into the callee. Its lifetime
        // is therefore the function body just like an owned local's, and
        // `[OWN-2]` requires the callee to destroy it on every exit unless it
        // was moved onward. Leaving parameters out of this list leaked every
        // owned argument whose type needed drop (D-043).
        let owned: Vec<LocalId> = function
            .params
            .iter()
            .enumerate()
            .filter(|(_, param)| {
                param.mode == hir::Mode::Owned && types.needs_drop(function.local(param.local).ty)
            })
            .map(|(index, _)| LocalId((index + 1) as u32))
            .collect();
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
            map,
            locals,
            blocks,
            local_map,
            current: BasicBlockId(0),
            loops: Vec::new(),
            defers: Vec::new(),
            owned,
            statement_temps: Vec::new(),
            arg_count,
            overflow: function.overflow,
            bool_ty: common.bool_,
            usize_ty: common.usize,
            void_ty: common.void,
            current_span: function.span,
        }
    }

    fn finish(self) -> Body {
        // `[FN-1]` — thread borrowed-ness to the move sites (D-041). A
        // borrowed parameter is a bitwise copy with no loan behind it, so the
        // move checker needs the mode written down: without it the callee
        // cannot tell borrowed data apart from owned data. Only `Borrow` is
        // listed (`Owned` takes ownership; `Mut` arrives as `ref mut`, whose
        // moves already carry `Deref`). Parameters occupy locals
        // `1..=arg_count` in order, so the HIR position maps directly.
        let borrowed_params: Vec<LocalId> = self
            .function
            .params
            .iter()
            .enumerate()
            .filter(|(_, p)| p.mode == hir::Mode::Borrow)
            .map(|(i, _)| LocalId((i + 1) as u32))
            .collect();
        Body {
            name: self.function.name.to_string(),
            symbol: self.function.symbol.clone(),
            locals: self.locals,
            blocks: self.blocks,
            arg_count: self.arg_count,
            span: self.function.span,
            borrows: self.function.borrows.clone(),
            borrowed_params,
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
        // `[DRP-3]` — "Temporaries drop at the end of the enclosing statement."
        // A temporary that owns something and is never registered is a leak
        // with no name to report it against: `R(1)` as a statement ran no
        // destructor at all, and a temporary holding an `Array[T]` leaked its
        // buffer. Registered here rather than at each construction site,
        // because every temporary comes through this one function and a site
        // that forgot would be invisible.
        if self.types.needs_drop(ty) {
            self.statement_temps.push(id);
        }
        id
    }

    /// `[DRP-3]` — drop the temporaries this statement made, last first.
    ///
    /// Separate from `emit_drops_from`, which is `[OWN-2]`'s *scope* end. The
    /// two differ in when, not in what: a named local lives to the end of its
    /// block, a temporary to the end of its statement. Running them through one
    /// list would give a temporary the wrong lifetime in either direction.
    fn emit_statement_temps(&mut self, mark: usize) {
        if self.statement_temps.len() <= mark {
            return;
        }
        let pending: Vec<LocalId> =
            self.statement_temps[mark..].iter().rev().copied().collect();
        for local in pending {
            self.push(StmtKind::Drop { place: Place::local(local), flag: None });
        }
        self.statement_temps.truncate(mark);
    }

    fn build(&mut self) {
        let body = &self.function.body;
        self.lower_block(body);
        // A function whose body falls off the end returns the (void) return
        // slot as it stands. `[FN-8]`'s `main` is the common case. Owned
        // parameters live outside the body block's local scope, so its
        // `lower_block` deliberately leaves them in `self.owned`; discharge
        // them here on the fallthrough path. Explicit `return` already calls
        // `emit_drops_from(0)` while lowering that statement.
        if matches!(self.blocks[self.current.0 as usize].terminator, Terminator::Unreachable) {
            self.emit_drops_from(0);
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
        let temps = self.statement_temps.len();
        self.lower_stmt_inner(stmt);
        // `[EXP-4]`, `[DRP-3]` — the statement is over, so its temporaries are.
        self.emit_statement_temps(temps);
    }

    fn lower_stmt_inner(&mut self, stmt: &'a hir::Stmt) {
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
                self.lower_assign(place, value);
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

    /// `[OWN-5]` — "Overwriting a place that holds a live value drops the old
    /// value first (after evaluating the new value: `x = f(x)` moves `x` into
    /// `f`, then stores)."
    ///
    /// The third of `[OWN-2]`'s three occasions to drop, and the one that was
    /// never built: scope end is `emit_drops_from`, statement end is
    /// `emit_statement_temps`, and an overwrite ran no destructor at all. A
    /// declared `drop` silently did not run and an `Array[T]` leaked its
    /// buffer, with the program's output identical either way — which is why
    /// `tests/conformance/OWN-5/` did not notice: its case moves the old value
    /// into the call that makes the new one, so nothing is live to drop.
    ///
    /// The order the rule gives is why the new value goes through a temporary:
    /// evaluate, *then* drop, *then* store. It also has to, because a value
    /// arriving from a call is written by a terminator, and a drop cannot be
    /// pushed between a terminator and itself.
    ///
    /// Which of these drops survives is left to `[OWN-3]`'s existing
    /// elaboration rather than decided here: a first initialisation finds the
    /// local `Moved` (`StorageLive` says so) and the drop is deleted, a
    /// conditionally-moved local gets a flag, and the temporary's own
    /// statement-end drop goes away because storing it here is a move.
    fn lower_assign(&mut self, place: Place, expr: &'a hir::Expr) {
        if !self.types.needs_drop(expr.ty) {
            self.lower_into(place, expr);
            return;
        }
        let temp = self.temp(expr.ty, expr.span);
        self.lower_into(Place::local(temp), expr);
        self.at(expr.span);
        self.push(StmtKind::Drop { place: place.clone(), flag: None });
        let value = self.read(Place::local(temp), expr.ty);
        self.push(StmtKind::Assign { place, rvalue: Rvalue::Use(value) });
    }

    /// A local that no scope and no statement will drop.
    ///
    /// `temp` registers what it makes with `[DRP-3]`, which is right for a
    /// temporary holding a value nobody else will claim. The cell operations
    /// need the opposite: a slot that holds a value on its way from one owner
    /// to another, where a second drop would be a double free.
    fn temp_unowned(&mut self, ty: Ty, span: ember_span::Span) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span });
        id
    }

    /// `[CELL-1]` — `c.set(v)` and `c.replace(v)`, which differ only in what
    /// becomes of the old value: `replace` hands it back, `set` drops it.
    ///
    /// **"`set` and `replace` MUST store the new value before dropping the old
    /// one."** The rule gives its own reason: a drop runs user code, that code
    /// can reach the same cell, and a drop-then-store implementation would let
    /// it read the cell while there is nothing in it. So the old value is moved
    /// out to a slot first, the new one is stored, and only then does the drop
    /// run — by which time the cell holds `v` and re-entering it is harmless.
    ///
    /// This is the one place in the compiler where that order is right, and it
    /// is exactly opposite to `[OWN-5]`, which every ordinary assignment
    /// follows. That is why `set` is not lowered as an assignment; ADR-020's
    /// last paragraph is the note not to generalise either rule to the other.
    ///
    /// The new value is lowered **first**, before the old one is disturbed:
    /// `update(f)` arrives here as `set(f(get()))`, and `f` has to see what the
    /// cell held.
    fn lower_cell_store(
        &mut self,
        old_into: Option<Place>,
        cell: &'a hir::Expr,
        value: &'a hir::Expr,
    ) {
        let inner = value.ty;
        let new = self.lower_operand(value);
        let field = self.lower_place(cell).field(0);
        self.at(cell.span);

        // Where the old value goes. `replace` was given a destination; `set`
        // needs a slot only if there is something to drop, and for a `T` that
        // owns nothing there is no slot and no drop — `[CELL-2]`'s "no overhead
        // relative to a plain field" is the ordinary case, and this is it.
        let old = match &old_into {
            Some(place) => Some(place.clone()),
            None if self.types.needs_drop(inner) => {
                Some(Place::local(self.temp_unowned(inner, cell.span)))
            }
            None => None,
        };
        if let Some(old) = &old {
            let taken = self.read(field.clone(), inner);
            self.push(StmtKind::Assign { place: old.clone(), rvalue: Rvalue::Use(taken) });
        }
        self.push(StmtKind::Assign { place: field, rvalue: Rvalue::Use(new) });
        // Only `set` drops. `replace`'s caller owns the old value now, so the
        // cell is never torn there at all.
        if old_into.is_none() {
            if let Some(old) = old {
                self.push(StmtKind::Drop { place: old, flag: None });
            }
        }
    }

    /// `[CELL-1]` — `c.into_inner()`, which takes `owned self`.
    ///
    /// The payload leaves and the cell must not be dropped behind it: the
    /// payload was the only thing it owned, so dropping the cell afterwards
    /// would free the value its caller now holds.
    ///
    /// Moving the *whole cell* into a slot first is what says so. A move out of
    /// a field is a partial move and `[OWN-3]`'s analysis does not track one,
    /// so it would leave the receiver looking live and drop it at scope end; a
    /// whole-local move is tracked, and marks the receiver moved. The slot
    /// itself is `temp_unowned`, so nothing drops it either — which is the
    /// point, because its field has left.
    fn lower_cell_into_inner(&mut self, place: Place, cell: &'a hir::Expr, inner: Ty) {
        let carrier = self.temp_unowned(cell.ty, cell.span);
        let whole = self.lower_place(cell);
        self.at(cell.span);
        let moved = self.read(whole, cell.ty);
        self.push(StmtKind::Assign {
            place: Place::local(carrier),
            rvalue: Rvalue::Use(moved),
        });
        let payload = self.read(Place::local(carrier).field(0), inner);
        self.push(StmtKind::Assign { place, rvalue: Rvalue::Use(payload) });
    }

    /// `[CELL-1]` — `c.update(f)`, which is `set(f(get()))`.
    ///
    /// `T: Copy` here, checked in typeck, so reading the old value is a load
    /// and leaves the cell intact: `f` runs with the cell still holding what it
    /// held, which is what makes it safe for `f` to reach the same cell. There
    /// is nothing to drop and no window to protect.
    fn lower_cell_update(&mut self, cell: &'a hir::Expr, f: &'a hir::Expr) {
        let field = self.lower_place(cell).field(0);
        let inner = self.types.kind(f.ty);
        let ret = match inner {
            TyKind::Fn { ret, .. } => *ret,
            // typeck checked the shape; a non-function here is an earlier error
            // already reported, and lowering has nothing useful to add.
            _ => return,
        };
        let callee = self.lower_operand(f);
        self.at(cell.span);
        let old = self.read(field.clone(), ret);
        let result = self.temp_unowned(ret, cell.span);
        let next = self.new_block();
        self.terminate(Terminator::Call {
            func: FuncRef::Indirect(callee),
            args: vec![old],
            dest: Place::local(result),
            next,
        });
        self.current = next;
        let produced = self.read(Place::local(result), ret);
        self.push(StmtKind::Assign { place: field, rvalue: Rvalue::Use(produced) });
    }

    /// `[CELL-5]`, `[CELL-7]`, `[CELL-9]` — `c.borrow()` / `c.borrow_mut()`.
    ///
    /// The `borrowed` argument is typeck's shared borrow of the cell (`&cell`),
    /// written down so `collect_loans` sees it: the guard's region borrows the
    /// cell, and `[TYP-15]` applies to the guard. Overlapping guards are
    /// allowed statically (all loans here are of the cell, all shared at this
    /// level); the counter refuses them at run time in every profile, and
    /// `exclusivity = "unchecked"` never reaches this path (there is no
    /// profile input to this function at all).
    ///
    /// Layout (see `refcell_of`): 0 `value: T`, 1 `borrow: isize` (`0`
    /// unborrowed, `>0` shared count, `-1` mutably borrowed), 2
    /// `borrow_file: *u8`, 3 `borrow_line: u32`.
    ///
    /// `borrow` fails iff the counter is `-1`; `borrow_mut` fails iff it is
    /// non-zero. Failure panics with the *stored* conflicting location via
    /// `AssertKind::RefCellBorrow`; success increments (`borrow`) or sets to
    /// `-1` (`borrow_mut`), stores the *current* location, and builds the
    /// guard (`Ref`/`RefMut` struct) from a borrow of `value`.
    fn lower_refcell_borrow(
        &mut self,
        place: Place,
        borrowed: &'a hir::Expr,
        mutable: bool,
        guard_ty: Ty,
        span: ember_span::Span,
    ) {
        let hir::ExprKind::Ref { place: cell_hir, .. } = &borrowed.kind else {
            return;
        };
        self.at(span);
        let cell_place = self.lower_place(cell_hir);
        let value_place = cell_place.clone().field(0);
        let borrow_place = cell_place.clone().field(1);
        let file_place = cell_place.clone().field(2);
        let line_place = cell_place.clone().field(3);

        let TyKind::Struct(guard_id) = *self.types.kind(guard_ty) else {
            return;
        };
        let ref_field_ty = self.types.struct_def(guard_id).fields[0].ty;

        // The loan the region graph sees: a borrow of the cell's value. For
        // `borrow_mut` this is a *mutable* loan, and overlapping mutable loans
        // from `RefCell`s are deliberately allowed statically — the counter
        // refuses them at run time. See `borrows.rs` (`is_refcell_loan` skips
        // Borrow conflicts for these but still rejects moves of the cell).
        let ref_temp = self.temp(ref_field_ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(ref_temp),
            rvalue: Rvalue::Ref { place: value_place.clone(), mutable },
        });
        let guard_operand = self.read(Place::local(ref_temp), ref_field_ty);

        let panic_bb = self.new_block();
        let success_bb = self.new_block();
        let counter = Operand::Copy(borrow_place.clone());
        if mutable {
            // `borrow_mut`: `0` succeeds, anything else panics.
            self.terminate(Terminator::SwitchInt {
                discr: counter,
                targets: vec![(0, success_bb)],
                otherwise: panic_bb,
            });
        } else {
            // `borrow`: `-1` (mutably borrowed) panics, anything else succeeds.
            self.terminate(Terminator::SwitchInt {
                discr: counter,
                targets: vec![(-1, panic_bb)],
                otherwise: success_bb,
            });
        }

        // Failure: panic with the stored conflicting location. The current
        // location travels as the assert's span (the backend renders it).
        self.current = panic_bb;
        self.at(span);
        let file_op = Operand::Copy(file_place.clone());
        let line_op = Operand::Copy(line_place.clone());
        let unreachable_bb = self.new_block();
        self.terminate(Terminator::Assert {
            cond: Operand::Const(Const::Bool(false)),
            expected: true,
            msg: AssertKind::RefCellBorrow { file: file_op, line: line_op },
            next: unreachable_bb,
            span,
        });
        self.current = unreachable_bb;
        self.terminate(Terminator::Unreachable);

        // Success: update the counter, store this borrow's location, build the
        // guard. The counter write targets field 1 while the loan is of field
        // 0 — distinct fields are disjoint (`[BRW-4]`), so the update never
        // conflicts with the loan that keeps the cell alive.
        self.current = success_bb;
        self.at(span);
        if mutable {
            // `-1` via `-(1)`: `Const::Int` holds a `u128` bit pattern, and a
            // huge positive would render as one in C. Negation is exact.
            let one_isize = {
                // The counter's own type, from the cell (field 1).
                let TyKind::Struct(cell_id) = *self.types.kind(cell_hir.ty) else {
                    return;
                };
                self.types.struct_def(cell_id).fields[1].ty
            };
            let neg_one = Rvalue::UnaryOp {
                op: crate::UnOp::Neg,
                operand: Operand::Const(Const::Int { value: 1, ty: one_isize }),
            };
            self.push(StmtKind::Assign { place: borrow_place.clone(), rvalue: neg_one });
        } else {
            let counter_ty = {
                let TyKind::Struct(cell_id) = *self.types.kind(cell_hir.ty) else {
                    return;
                };
                self.types.struct_def(cell_id).fields[1].ty
            };
            let one = Operand::Const(Const::Int { value: 1, ty: counter_ty });
            self.push(StmtKind::Assign {
                place: borrow_place.clone(),
                rvalue: Rvalue::BinaryOp {
                    op: BinOp::Add,
                    lhs: Operand::Copy(borrow_place.clone()),
                    rhs: one,
                },
            });
        }
        let (path, line) = self.borrow_location(span);
        self.push(StmtKind::Assign {
            place: file_place,
            rvalue: Rvalue::Use(Operand::Const(Const::CStr(path))),
        });
        let line_ty = {
            let TyKind::Struct(cell_id) = *self.types.kind(cell_hir.ty) else {
                return;
            };
            self.types.struct_def(cell_id).fields[3].ty
        };
        self.push(StmtKind::Assign {
            place: line_place,
            rvalue: Rvalue::Use(Operand::Const(Const::Int { value: line as u128, ty: line_ty })),
        });
        self.push(StmtKind::Assign {
            place: place.clone(),
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Struct(guard_id),
                operands: vec![guard_operand],
            },
        });
        let join_bb = self.new_block();
        self.terminate(Terminator::Goto(join_bb));
        self.current = join_bb;
    }

    /// `[CELL-6]`, `[CELL-6a]` — `c.try_borrow()` / `c.try_borrow_mut()`.
    ///
    /// As above, except contention builds `None` rather than panicking. There
    /// is no profile input here either, so no profile can make these
    /// infallible (`[PRF-1]` forbids changing which `match` arm runs).
    fn lower_refcell_try_borrow(
        &mut self,
        place: Place,
        borrowed: &'a hir::Expr,
        mutable: bool,
        opt_ty: Ty,
        span: ember_span::Span,
    ) {
        let hir::ExprKind::Ref { place: cell_hir, .. } = &borrowed.kind else {
            return;
        };
        self.at(span);
        let cell_place = self.lower_place(cell_hir);
        let value_place = cell_place.clone().field(0);
        let borrow_place = cell_place.clone().field(1);
        let file_place = cell_place.clone().field(2);
        let line_place = cell_place.clone().field(3);

        let TyKind::Enum(opt_id) = *self.types.kind(opt_ty) else {
            return;
        };
        let none_index = self
            .types
            .enum_def(opt_id)
            .variants
            .iter()
            .position(|v| v.fields.is_empty())
            .unwrap_or(0);
        let some_index = 1 - none_index;
        let guard_ty = self.types.enum_def(opt_id).variants[some_index].fields[0].ty;
        let TyKind::Struct(guard_id) = *self.types.kind(guard_ty) else {
            return;
        };
        let ref_field_ty = self.types.struct_def(guard_id).fields[0].ty;

        let ref_temp = self.temp(ref_field_ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(ref_temp),
            rvalue: Rvalue::Ref { place: value_place.clone(), mutable },
        });
        let guard_operand = self.read(Place::local(ref_temp), ref_field_ty);

        let success_bb = self.new_block();
        let none_bb = self.new_block();
        let join_bb = self.new_block();
        let counter = Operand::Copy(borrow_place.clone());
        if mutable {
            self.terminate(Terminator::SwitchInt {
                discr: counter,
                targets: vec![(0, success_bb)],
                otherwise: none_bb,
            });
        } else {
            self.terminate(Terminator::SwitchInt {
                discr: counter,
                targets: vec![(-1, none_bb)],
                otherwise: success_bb,
            });
        }

        self.current = none_bb;
        self.at(span);
        self.push(StmtKind::Assign {
            place: place.clone(),
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Enum(opt_id, none_index),
                operands: Vec::new(),
            },
        });
        self.terminate(Terminator::Goto(join_bb));

        self.current = success_bb;
        self.at(span);
        if mutable {
            let one_isize = {
                let TyKind::Struct(cell_id) = *self.types.kind(cell_hir.ty) else {
                    return;
                };
                self.types.struct_def(cell_id).fields[1].ty
            };
            let neg_one = Rvalue::UnaryOp {
                op: crate::UnOp::Neg,
                operand: Operand::Const(Const::Int { value: 1, ty: one_isize }),
            };
            self.push(StmtKind::Assign { place: borrow_place.clone(), rvalue: neg_one });
        } else {
            let counter_ty = {
                let TyKind::Struct(cell_id) = *self.types.kind(cell_hir.ty) else {
                    return;
                };
                self.types.struct_def(cell_id).fields[1].ty
            };
            let one = Operand::Const(Const::Int { value: 1, ty: counter_ty });
            self.push(StmtKind::Assign {
                place: borrow_place.clone(),
                rvalue: Rvalue::BinaryOp {
                    op: BinOp::Add,
                    lhs: Operand::Copy(borrow_place.clone()),
                    rhs: one,
                },
            });
        }
        let (path, line) = self.borrow_location(span);
        self.push(StmtKind::Assign {
            place: file_place,
            rvalue: Rvalue::Use(Operand::Const(Const::CStr(path))),
        });
        let line_ty = {
            let TyKind::Struct(cell_id) = *self.types.kind(cell_hir.ty) else {
                return;
            };
            self.types.struct_def(cell_id).fields[3].ty
        };
        self.push(StmtKind::Assign {
            place: line_place,
            rvalue: Rvalue::Use(Operand::Const(Const::Int { value: line as u128, ty: line_ty })),
        });
        let guard_temp = self.temp_unowned(guard_ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(guard_temp),
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Struct(guard_id),
                operands: vec![guard_operand],
            },
        });
        let some_operand = self.read(Place::local(guard_temp), guard_ty);
        self.push(StmtKind::Assign {
            place,
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Enum(opt_id, some_index),
                operands: vec![some_operand],
            },
        });
        self.terminate(Terminator::Goto(join_bb));
        self.current = join_bb;
    }

    /// The current borrow's source location for the cell's location fields:
    /// `(path, line)`. Paths use `/` separators so the panic reads the same
    /// on every host. A dummy span (synthesised nodes) stores no location.
    fn borrow_location(&self, span: ember_span::Span) -> (String, u32) {
        if span.is_dummy() {
            return (String::new(), 0);
        }
        let Some(file) = self.map.get(span.file) else {
            return (String::new(), 0);
        };
        let path = file.path.display().to_string().replace('\\', "/");
        let line = file.line_col(span.start).line;
        (path, line)
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
            // `[RNG-3]` — `T.checked(v) -> Result[T, RangeError]`. Two
            // compares and a branch, building `Ok(v)` or `Err(OutOfRange)`.
            // It is lowered here rather than in the backend because it
            // produces an enum value, which is a branch and two aggregates
            // rather than a C expression.
            // `[SPN-2]` — `s.get(i) -> Option[ref T]`, "the
            // checked-without-panic form". One compare, a branch, and the two
            // `Option` variants.
            // `[CLO-3]` — a call through a value of function type.
            hir::ExprKind::CallIndirect { callee, args } => {
                let callee_op = self.lower_operand(callee);
                let args: Vec<Operand> =
                    args.iter().map(|a| self.lower_operand_borrowed(a)).collect();
                let next = self.new_block();
                self.terminate(Terminator::Call {
                    func: FuncRef::Indirect(callee_op),
                    args,
                    dest: place,
                    next,
                });
                self.current = next;
            }
            hir::ExprKind::Builtin { which: hir::Builtin::SpanGet, args } => {
                self.lower_span_get(place, &args[0], &args[1], expr.ty, expr.span);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::CellSet, args } => {
                self.lower_cell_store(None, &args[0], &args[1]);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::CellReplace, args } => {
                self.lower_cell_store(Some(place), &args[0], &args[1]);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::CellIntoInner, args } => {
                self.lower_cell_into_inner(place, &args[0], expr.ty);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::CellUpdate, args } => {
                self.lower_cell_update(&args[0], &args[1]);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::RefCellBorrow, args } => {
                self.lower_refcell_borrow(place, &args[0], false, expr.ty, expr.span);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::RefCellBorrowMut, args } => {
                self.lower_refcell_borrow(place, &args[0], true, expr.ty, expr.span);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::RefCellTryBorrow, args } => {
                self.lower_refcell_try_borrow(place, &args[0], false, expr.ty, expr.span);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::RefCellTryBorrowMut, args } => {
                self.lower_refcell_try_borrow(place, &args[0], true, expr.ty, expr.span);
            }
            hir::ExprKind::Builtin { which: hir::Builtin::RangeChecked(id), args } => {
                self.lower_range_checked(place, *id, &args[0], expr.ty, expr.span);
            }
            hir::ExprKind::Builtin { which, args } => {
                // `arg_ty` is what the backend picks its implementation from.
                // For most builtins that is the first argument; `alloc` takes
                // a count and returns the pointer, so its element type is in
                // the result, and `size_of` carries it on a placeholder.
                let arg_ty = match which {
                    hir::Builtin::MemAlloc
                    | hir::Builtin::ArenaWithCapacity => expr.ty,
                    hir::Builtin::SizeOf => args.last().map(|a| a.ty).unwrap_or(expr.ty),
                    _ => args.first().map(|a| a.ty).unwrap_or(expr.ty),
                };
                // `push` copies the value through a pointer, so a constant
                // has to live somewhere addressable: `&10` is not C. Anything
                // with a place keeps the shape `lower_operand` gives it:
                // `push` takes ownership (`[OWN-3]` — moves transfer
                // ownership; the source is dead), so a move stays a move and
                // `[OWN-3]`'s elaboration deletes the temporary's
                // statement-end drop. Erasing it to a copy destroys the value
                // twice — once as the temporary, once with the buffer.
                let spill = matches!(
                    which,
                    hir::Builtin::ArrayPush
                        | hir::Builtin::ArenaAlloc { .. }
                        | hir::Builtin::FixedArenaAlloc { .. }
                        | hir::Builtin::ScopedArenaAlloc { .. }
                );
                let args: Vec<Operand> = args
                    .iter()
                    .enumerate()
                    .map(|(index, a)| {
                        if spill && index == 1 {
                            match self.lower_operand(a) {
                                Operand::Const(_) => self.lower_into_temp(a),
                                other => other,
                            }
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
    /// `[SPN-2]` — `s.get(i)`.
    ///
    /// `Some(ref s[i])` where `i < s.len()`, `None` otherwise. Unlike `s[i]`
    /// this emits no `Assert`: the comparison **is** the answer, which is what
    /// "checked without panic" means.
    fn lower_span_get(
        &mut self,
        place: Place,
        receiver: &'a hir::Expr,
        index: &'a hir::Expr,
        result_ty: Ty,
        span: ember_span::Span,
    ) {
        let base = self.lower_place(receiver);
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
                rhs: Operand::Copy(base.clone().field(1)),
            },
        });

        let TyKind::Enum(option) = *self.types.kind(result_ty) else {
            unreachable!("[SPN-2] `get` returns an Option");
        };
        // `option_of` builds `None` first and `Some` second, so `None` is
        // variant 0 — which is also what makes `Option`'s discriminant zero
        // for the absent case. Read from the table rather than assumed: the
        // order is a choice in one function, not a language rule.
        let none_index = self
            .types
            .enum_def(option)
            .variants
            .iter()
            .position(|v| v.fields.is_empty())
            .expect("an Option has a payload-free variant");
        let some_index = 1 - none_index;
        let some_bb = self.new_block();
        let none_bb = self.new_block();
        let join_bb = self.new_block();
        self.terminate(Terminator::SwitchInt {
            discr: Operand::Copy(Place::local(in_range)),
            targets: vec![(0, none_bb)],
            otherwise: some_bb,
        });

        self.current = some_bb;
        let element = base.index(slot);
        let reference =
            self.temp(self.types.enum_def(option).variants[some_index].fields[0].ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(reference),
            rvalue: Rvalue::Ref { place: element, mutable: false },
        });
        self.push(StmtKind::Assign {
            place: place.clone(),
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Enum(option, some_index),
                operands: vec![Operand::Move(Place::local(reference))],
            },
        });
        self.terminate(Terminator::Goto(join_bb));

        self.current = none_bb;
        self.push(StmtKind::Assign {
            place,
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Enum(option, none_index),
                operands: Vec::new(),
            },
        });
        self.terminate(Terminator::Goto(join_bb));

        self.current = join_bb;
    }

    /// `[RNG-3]` — `T.checked(v)`.
    ///
    /// `in_range = v >= lo and v <= hi` (or `v < hi` for a half-open range),
    /// then `Ok(v)` or `Err(RangeError.OutOfRange)`. `[RNG-6]` falls out: NaN
    /// fails both comparisons, so it lands in `Err`, which is what "NaN is in
    /// no range" means at the construction site.
    fn lower_range_checked(
        &mut self,
        place: Place,
        id: ember_types::RangeId,
        value: &'a hir::Expr,
        result_ty: Ty,
        span: ember_span::Span,
    ) {
        let def = self.types.range_def(id).clone();
        let repr = def.repr;
        let value = self.lower_operand(value);

        let lo = self.bound_operand(def.lo, repr);
        let hi = self.bound_operand(def.hi, repr);

        let above_lo = self.temp(self.bool_ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(above_lo),
            rvalue: Rvalue::BinaryOp { op: BinOp::Ge, lhs: value.clone(), rhs: lo },
        });
        let below_hi = self.temp(self.bool_ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(below_hi),
            rvalue: Rvalue::BinaryOp {
                op: if def.inclusive { BinOp::Le } else { BinOp::Lt },
                lhs: value.clone(),
                rhs: hi,
            },
        });
        let in_range = self.temp(self.bool_ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(in_range),
            rvalue: Rvalue::BinaryOp {
                op: BinOp::BitAnd,
                lhs: Operand::Copy(Place::local(above_lo)),
                rhs: Operand::Copy(Place::local(below_hi)),
            },
        });

        // `Result[T, RangeError]` — `Ok` is variant 0 and `Err` variant 1, in
        // the order `result_of` builds them.
        let TyKind::Enum(result_enum) = *self.types.kind(result_ty) else {
            unreachable!("[RNG-3] `checked` returns a Result enum");
        };

        let ok_bb = self.new_block();
        let err_bb = self.new_block();
        let join_bb = self.new_block();
        self.terminate(Terminator::SwitchInt {
            discr: Operand::Copy(Place::local(in_range)),
            targets: vec![(0, err_bb)],
            otherwise: ok_bb,
        });

        self.current = ok_bb;
        self.push(StmtKind::Assign {
            place: place.clone(),
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Enum(result_enum, 0),
                operands: vec![value],
            },
        });
        self.terminate(Terminator::Goto(join_bb));

        self.current = err_bb;
        // `RangeError` has one unit variant, so the payload is empty.
        let TyKind::Enum(err_enum) =
            *self.types.kind(self.types.enum_def(result_enum).variants[1].fields[0].ty)
        else {
            unreachable!("[RNG-3] the error half is `RangeError`");
        };
        let err_value = self.temp(self.types.enum_def(result_enum).variants[1].fields[0].ty, span);
        self.push(StmtKind::Assign {
            place: Place::local(err_value),
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Enum(err_enum, 0),
                operands: Vec::new(),
            },
        });
        self.push(StmtKind::Assign {
            place,
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Enum(result_enum, 1),
                operands: vec![Operand::Move(Place::local(err_value))],
            },
        });
        self.terminate(Terminator::Goto(join_bb));

        self.current = join_bb;
    }

    /// A range endpoint as a MIR operand of the representation type.
    fn bound_operand(&mut self, bound: ember_types::Bound, repr: Ty) -> Operand {
        match bound {
            ember_types::Bound::Int(v) => {
                // A negative endpoint is stored as the two's-complement bit
                // pattern, which is what `Const::Int`'s `u128` holds.
                Operand::Const(Const::Int { value: v as u128, ty: repr })
            }
            ember_types::Bound::Float(v) => Operand::Const(Const::Float { value: v, ty: repr }),
        }
    }

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
            // `[FN-6]` — a named function as a value: its symbol.
            hir::ExprKind::FnValue(def) => {
                Rvalue::Use(Operand::Const(Const::Fn(self.program.function(*def).symbol.clone())))
            }
            hir::ExprKind::Widen { expr: inner, to } => {
                let operand = self.lower_operand(inner);
                Rvalue::Cast { kind: CastKind::Widen, operand, to: *to }
            }
            // `[TYP-5]` range erasure. `[COST-3]` classes a range type as
            // **not observable**: it is its representation's bits, and the
            // check lives at the construction site. So the node disappears
            // here rather than lowering to a conversion — there is nothing to
            // convert.
            hir::ExprKind::EraseRange(inner) => Rvalue::Use(self.lower_operand(inner)),
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
            // An `Array[T]`'s length is a field, read at the point of use;
            // a view's is the same field of the same shape (`[SPN-2]`:
            // "Indexing a `Span` is bounds-checked").
            TyKind::Vec { .. } | TyKind::Span { .. } => None,
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
        // Kept for the projection below; the slot is still built and still
        // bounds-checked, because a constant index into a dynamically sized
        // container is still an index that can be out of range.
        let index_const = match &index_op {
            Operand::Const(Const::Int { value, .. }) => u64::try_from(*value).ok(),
            _ => None,
        };
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

        // `[BRW-5]` — "`ref mut a[i]` and `ref mut a[j]` conflict **unless
        // both indices are constants and different**." That exemption is
        // decided by `overlaps`, which compares `ConstIndex` projections — and
        // it was unreachable, because every index went into a runtime slot and
        // came back as `Index(local)`. Two borrows of `v[0]` and `v[1]` were
        // rejected as though the indices might be equal, which the rule says
        // they may not be.
        //
        // The bounds check above is emitted either way and is not the question:
        // for an `Array[T]` the *length* is dynamic even when the index is a
        // literal, so the check still has to run. What the constant buys is
        // disjointness, not the elision of a check.
        match index_const {
            Some(value) => base.const_index(value),
            None => base.index(slot),
        }
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
