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
use ember_types::{Ty, TypeTable};

use crate::{
    AggregateKind, BasicBlock, BasicBlockId, Body, CastKind, Const, FuncRef, LocalDecl, LocalId,
    LocalKind, Operand, Place, RETURN_LOCAL, Rvalue, Stmt, Terminator,
};

pub fn lower(program: &hir::Program, types: &TypeTable) -> Vec<Body> {
    program.functions.iter().map(|f| lower_function(f, program, types)).collect()
}

fn lower_function(function: &hir::Function, program: &hir::Program, types: &TypeTable) -> Body {
    let mut builder = Builder::new(function, program, types);
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

fn successors(terminator: &Terminator) -> Vec<BasicBlockId> {
    match terminator {
        Terminator::Goto(bb) => vec![*bb],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut out: Vec<BasicBlockId> = targets.iter().map(|(_, bb)| *bb).collect();
            out.push(*otherwise);
            out
        }
        Terminator::Call { next, .. } => vec![*next],
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
        Terminator::Call { next, .. } => fix(next),
        Terminator::Return | Terminator::Unreachable => {}
    }
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
    /// The loop stack: (continue target, break target).
    loops: Vec<(BasicBlockId, BasicBlockId)>,
    arg_count: usize,
}

impl<'a> Builder<'a> {
    fn new(function: &'a hir::Function, program: &'a hir::Program, types: &'a TypeTable) -> Builder<'a> {
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

        let blocks = vec![BasicBlock { stmts: Vec::new(), terminator: Terminator::Unreachable }];
        Builder {
            function,
            program,
            types,
            locals,
            blocks,
            local_map,
            current: BasicBlockId(0),
            loops: Vec::new(),
            arg_count,
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
        }
    }

    // -- block construction --------------------------------------------------

    fn new_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.blocks.len() as u32);
        self.blocks.push(BasicBlock { stmts: Vec::new(), terminator: Terminator::Unreachable });
        id
    }

    fn push(&mut self, stmt: Stmt) {
        self.blocks[self.current.0 as usize].stmts.push(stmt);
    }

    fn terminate(&mut self, terminator: Terminator) {
        self.blocks[self.current.0 as usize].terminator = terminator;
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

    fn lower_block(&mut self, block: &hir::Block) {
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
        }
    }

    fn lower_stmt(&mut self, stmt: &hir::Stmt) {
        match stmt {
            hir::Stmt::Let { local, init } => {
                let mir_local = self.local_map[local.0 as usize];
                self.push(Stmt::StorageLive(mir_local));
                if let Some(init) = init {
                    let place = Place::local(mir_local);
                    self.lower_into(place, init);
                }
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
            hir::Stmt::While { cond, body } => {
                let head_bb = self.new_block();
                let body_bb = self.new_block();
                let exit_bb = self.new_block();
                self.terminate(Terminator::Goto(head_bb));

                self.current = head_bb;
                let discr = self.lower_operand(cond);
                self.terminate(Terminator::SwitchInt {
                    discr,
                    targets: vec![(0, exit_bb)],
                    otherwise: body_bb,
                });

                self.loops.push((head_bb, exit_bb));
                self.current = body_bb;
                self.lower_block(body);
                self.goto_if_open(head_bb);
                self.loops.pop();

                self.current = exit_bb;
            }
            hir::Stmt::Block(block) => self.lower_block(block),
            hir::Stmt::Break => {
                if let Some(&(_, exit)) = self.loops.last() {
                    self.terminate(Terminator::Goto(exit));
                    self.current = self.new_block();
                }
            }
            hir::Stmt::Continue => {
                if let Some(&(head, _)) = self.loops.last() {
                    self.terminate(Terminator::Goto(head));
                    self.current = self.new_block();
                }
            }
        }
    }

    /// Terminate the current block with a jump, unless it already ended.
    fn goto_if_open(&mut self, target: BasicBlockId) {
        if matches!(self.blocks[self.current.0 as usize].terminator, Terminator::Unreachable) {
            self.terminate(Terminator::Goto(target));
        }
    }

    // -- expressions ----------------------------------------------------------

    /// Lower `expr` so that its value ends up in `place`.
    fn lower_into(&mut self, place: Place, expr: &hir::Expr) {
        match &expr.kind {
            hir::ExprKind::Call { callee, args } => {
                let function = self.program.function(*callee);
                let symbol = function.symbol.clone();
                let args: Vec<Operand> = args.iter().map(|a| self.lower_operand(a)).collect();
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
                let arg_ty = args.first().map(|a| a.ty).unwrap_or(expr.ty);
                let args: Vec<Operand> = args.iter().map(|a| self.lower_operand(a)).collect();
                let next = self.new_block();
                self.terminate(Terminator::Call {
                    func: FuncRef::Builtin { which: *which, arg_ty },
                    args,
                    dest: place,
                    next,
                });
                self.current = next;
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
                self.push(Stmt::Assign { place: place.clone(), rvalue: Rvalue::Use(rhs_operand) });
                self.terminate(Terminator::Goto(join_bb));

                self.current = short_bb;
                let short_value = Const::Bool(*op == hir::BinOp::Or);
                self.push(Stmt::Assign {
                    place: place.clone(),
                    rvalue: Rvalue::Use(Operand::Const(short_value)),
                });
                self.terminate(Terminator::Goto(join_bb));

                self.current = join_bb;
            }
            _ => {
                let rvalue = self.lower_rvalue(expr);
                self.push(Stmt::Assign { place, rvalue });
            }
        }
    }

    fn lower_rvalue(&mut self, expr: &hir::Expr) -> Rvalue {
        match &expr.kind {
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
            _ => Rvalue::Use(self.lower_operand(expr)),
        }
    }

    fn lower_operand(&mut self, expr: &hir::Expr) -> Operand {
        match &expr.kind {
            hir::ExprKind::Int(value) => Operand::Const(Const::Int { value: *value, ty: expr.ty }),
            hir::ExprKind::Float(value) => {
                Operand::Const(Const::Float { value: *value, ty: expr.ty })
            }
            hir::ExprKind::Bool(value) => Operand::Const(Const::Bool(*value)),
            hir::ExprKind::Str(text) => Operand::Const(Const::Str(text.clone())),
            hir::ExprKind::Error => Operand::Const(Const::Void),
            hir::ExprKind::Local(_) | hir::ExprKind::Field { .. } => {
                let place = self.lower_place(expr);
                self.read(place, expr.ty)
            }
            // Anything else needs a temporary to hold its value.
            _ => {
                let temp = self.temp(expr.ty, expr.span);
                self.push(Stmt::StorageLive(temp));
                self.lower_into(Place::local(temp), expr);
                self.read(Place::local(temp), expr.ty)
            }
        }
    }

    /// `[MIR-2]` — a non-`Copy` place is moved, not copied.
    fn read(&self, place: Place, ty: Ty) -> Operand {
        if self.types.is_copy(ty) { Operand::Copy(place) } else { Operand::Move(place) }
    }

    fn lower_place(&mut self, expr: &hir::Expr) -> Place {
        match &expr.kind {
            hir::ExprKind::Local(local) => Place::local(self.local_map[local.0 as usize]),
            hir::ExprKind::Field { base, index } => self.lower_place(base).field(*index),
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
