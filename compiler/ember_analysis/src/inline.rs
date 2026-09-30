//! Inlining in the MIR, so the analyses after it see across a call.
//!
//! A function called from exactly one place is inlined there: the program
//! does the same work with no call, the code does not grow, and what the
//! caller knows reaches the callee's body. Range facts (`[RNG-4]`) then know
//! a lambda's argument is the loop counter that was passed, and remove the
//! checks that cannot fail; the C compiler alone cannot, since the checks
//! are already written. Each copy `[CLO-3]` makes for a function argument
//! has one caller, so it and the function it calls fold into the caller.
//!
//! The callee's locals and blocks are appended to the caller's, its
//! parameters assigned from the arguments, and each `return` becomes a jump
//! back with the result moved into the call's destination. Everything else is
//! the callee's MIR unchanged, spans included, so a panic still names the
//! callee's line. A borrowed argument is handed over as the bits it is, as
//! the call passed it, with no retain; an owned one exactly as the call would
//! have. Functions whose call boundary carries meaning of its own stay calls:
//! methods (access intervals and dispatch), closures with an environment,
//! access transfers, `[FN-5]` forwarding, hoisted loop intervals, foreign and
//! exported functions, and anything recursive.

use std::collections::{BTreeMap, HashMap, HashSet};

use ember_types::{TyKind, TypeTable};

use ember_mir::{
    BasicBlock, BasicBlockId, Body, FuncRef, LocalId, Operand, ParameterMode, Place, Projection, Rvalue, Stmt,
    StmtKind, Terminator,
};

/// Inline every function called from exactly one place into that place;
/// returns how many calls were inlined. With `only_checked`, only a function
/// holding a check (a runtime assertion or a checked operation): the reason
/// to inline one before the C compiler does is that the caller's facts can
/// remove its checks (ADR-081). A function with none is left to a C compiler
/// that inlines as well itself (clang, gcc): given an inlined generic
/// `larger`, clang picks the slower unsigned `max`.
pub fn inline_single_calls_all(bodies: &mut Vec<Body>, types: &TypeTable, only_checked: bool) -> usize {
    let mut inlined = 0;
    let mut folded: HashSet<String> = HashSet::new();
    // Bottom-up, in a fixed order: a function is inlined only once nothing
    // it calls is still waiting to be, so its body is final when copied (a
    // copy made earlier would leave its callees a second call site), and the
    // same program always gives the same C. Repeat until nothing changes.
    loop {
        let index: HashMap<String, usize> =
            bodies.iter().enumerate().map(|(i, body)| (body.symbol.clone(), i)).collect();
        let mut sites: BTreeMap<String, Vec<(usize, usize)>> = BTreeMap::new();
        for (caller, body) in bodies.iter().enumerate() {
            for (block, data) in body.blocks.iter().enumerate() {
                if let Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } = &data.terminator {
                    sites.entry(symbol.clone()).or_default().push((caller, block));
                }
            }
        }
        // Each function with one call site, not in that site's own body, and
        // not at a call an `[FN-5]` binding names (it forwards an argument
        // into that call).
        let candidates: BTreeMap<&String, (usize, usize, usize)> = sites
            .iter()
            .filter_map(|(symbol, places)| {
                let [(caller, block)] = places.as_slice() else { return None };
                if bodies[*caller].call_argument_bindings.iter().any(|binding| binding.call_block.0 as usize == *block) {
                    return None;
                }
                let &callee = index.get(symbol)?;
                (callee != *caller
                    && inlinable(&bodies[callee], types)
                    && !calls_itself(&bodies[callee])
                    && (!only_checked || holds_a_check(&bodies[callee])))
                    .then_some((symbol, (*caller, *block, callee)))
            })
            .collect();
        let waiting: HashSet<&str> = candidates.keys().map(|symbol| symbol.as_str()).collect();
        let ready: Vec<(usize, usize, usize)> = candidates
            .values()
            .filter(|(_, _, callee)| {
                !bodies[*callee].blocks.iter().any(|data| {
                    matches!(&data.terminator, Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. }
                        if waiting.contains(symbol.as_str()))
                })
            })
            .copied()
            .collect();
        if ready.is_empty() {
            break;
        }
        // A ready callee has no waiting calls, so nothing is inlined into it
        // this round, and a caller's other call blocks keep their numbers.
        for (caller, block, callee) in ready {
            let callee_body = bodies[callee].clone();
            inline_call(&mut bodies[caller], block, &callee_body);
            folded.insert(callee_body.symbol);
            inlined += 1;
        }
    }
    // An inlined function nothing else names is not emitted: no call, no
    // function value, no interface table refers to it any more.
    let named = named_symbols(bodies);
    bodies.retain(|body| !folded.contains(&body.symbol) || named.contains(&body.symbol));
    inlined
}

/// Whether a body holds a check the range facts could remove: an assertion
/// (bounds, overflow, division by zero, ...) or a checked operation.
fn holds_a_check(body: &Body) -> bool {
    body.blocks.iter().any(|data| {
        matches!(data.terminator, Terminator::Assert { .. })
            || data.stmts.iter().any(|stmt| matches!(stmt.kind, StmtKind::CheckedBinaryOp { .. }))
    })
}

/// Every symbol a body names: calls, function values, interface adapters.
fn named_symbols(bodies: &[Body]) -> HashSet<String> {
    let mut named = HashSet::new();
    let operand = |operand: &Operand, named: &mut HashSet<String>| {
        if let Operand::Const(ember_mir::Const::Fn(symbol)) = operand {
            named.insert(symbol.clone());
        }
    };
    for body in bodies {
        for block in &body.blocks {
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Assign { rvalue, .. } => match rvalue {
                        Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => {
                            operand(value, &mut named)
                        }
                        Rvalue::Repeat { value, .. } => operand(value, &mut named),
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            operand(lhs, &mut named);
                            operand(rhs, &mut named);
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            for value in operands {
                                operand(value, &mut named);
                            }
                        }
                        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
                    },
                    StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                        operand(lhs, &mut named);
                        operand(rhs, &mut named);
                    }
                    _ => {}
                }
            }
            if let Terminator::Call { func, args, .. } = &block.terminator {
                for arg in args {
                    operand(arg, &mut named);
                }
                match func {
                    FuncRef::Direct { symbol, .. } => {
                        named.insert(symbol.clone());
                    }
                    FuncRef::Indirect { operand: callee, .. } => operand(callee, &mut named),
                    FuncRef::DynBoxNew { implementations, .. } => {
                        named.extend(implementations.iter().flatten().map(|method| method.symbol.clone()));
                    }
                    _ => {}
                }
            }
        }
    }
    named
}

/// Whether a function's call boundary carries nothing but the call.
fn inlinable(body: &Body, types: &TypeTable) -> bool {
    body.abi.is_none()
        && !body.is_extern_declaration
        && !body.is_abstract
        && body.class_owner.is_none()
        && body.class_virtual_slot.is_none()
        && body.closure_environment.is_none()
        && !body.mut_self
        && body.hoisted_accesses.is_empty()
        && body.call_argument_bindings.is_empty()
        && body.ffi_counted.is_none()
        && !body.emit_if_used
        // A method (its first parameter `self`) may be named by an interface
        // table or drop glue, which find it by its type, not by a call.
        && !(body.arg_count > 0 && body.locals.get(1).and_then(|l| l.name.as_deref()) == Some("self"))
        // A body keeps per-parameter state for interface calls (its
        // itable cache), which inlining would drop.
        && !body.locals.iter().skip(1).take(body.arg_count).any(|decl| {
            matches!(types.kind(decl.ty), TyKind::ClassInterface(_) | TyKind::Dyn { .. })
                || matches!(types.kind(decl.ty), TyKind::Ref { inner, .. } if matches!(types.kind(*inner), TyKind::ClassInterface(_) | TyKind::Dyn { .. }))
        })
        && body.symbol != ember_branding::mangled("main")
        // Dynamic access checks stay with their function: each is reported
        // under it (`[EFF-10]`), and its interval is the call's.
        && !body.blocks.iter().any(|block| {
            block.stmts.iter().any(|stmt| {
                matches!(
                    stmt.kind,
                    StmtKind::BeginAccess { .. }
                        | StmtKind::EndAccess { .. }
                        | StmtKind::BeginAccessTransfer { .. }
                        | StmtKind::EndAccessTransfer { .. }
                )
            })
        })
}

fn calls_itself(body: &Body) -> bool {
    body.blocks.iter().any(|block| {
        matches!(&block.terminator, Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } if *symbol == body.symbol)
    })
}

/// Replace the call ending `block` in `caller` with `callee`'s body.
fn inline_call(caller: &mut Body, block: usize, callee: &Body) {
    let Terminator::Call { args, dest, next, .. } = caller.blocks[block].terminator.clone() else {
        unreachable!("an inlined site is a call")
    };
    let span = caller.blocks[block].terminator_span;
    let local_base = caller.locals.len() as u32;
    let block_base = caller.blocks.len() as u32;
    let local = |id: LocalId| LocalId(id.0 + local_base);
    let target = |id: BasicBlockId| BasicBlockId(id.0 + block_base);

    // The callee's locals, its return slot and parameters among them.
    for decl in &callee.locals {
        let mut decl = decl.clone();
        decl.kind = ember_mir::LocalKind::Temp;
        caller.locals.push(decl);
    }
    // The call becomes: parameters set from the arguments, then a jump in.
    let mut stmts = Vec::new();
    for (k, arg) in args.iter().enumerate() {
        let param = Place::local(local(LocalId(k as u32 + 1)));
        // A borrowed argument is the caller's value lent for the call: the
        // parameter holds its bits with no count of its own, retained on the
        // way in no more than the callee releases it on the way out.
        if callee.param_modes.get(k) == Some(&ParameterMode::Borrow) {
            caller.uncounted_handles.push(param.local);
        }
        let arg = arg.clone();
        stmts.push(Stmt::new(StmtKind::StorageLive(param.local), span));
        stmts.push(Stmt::new(StmtKind::Assign { place: param, rvalue: Rvalue::Use(arg) }, span));
    }
    stmts.push(Stmt::new(StmtKind::StorageLive(local(LocalId(0))), span));
    caller.blocks[block].stmts.extend(stmts);
    caller.blocks[block].terminator = Terminator::Goto(target(BasicBlockId(0)));

    // The callee's blocks, renumbered; a return moves the result out and
    // jumps to where the call would have returned.
    for data in &callee.blocks {
        let mut data: BasicBlock = data.clone();
        for stmt in &mut data.stmts {
            renumber_stmt(stmt, &local);
        }
        renumber_terminator(&mut data.terminator, &local, &target);
        if matches!(data.terminator, Terminator::Return) {
            data.stmts.push(Stmt::new(
                StmtKind::Assign { place: dest.clone(), rvalue: Rvalue::Use(Operand::Move(Place::local(local(LocalId(0))))) },
                data.terminator_span,
            ));
            data.stmts.push(Stmt::new(StmtKind::StorageDead(local(LocalId(0))), data.terminator_span));
            data.terminator = Terminator::Goto(next);
        }
        caller.blocks.push(data);
    }
    // The callee's records, in the caller's numbering.
    caller.removed_checks.extend(callee.removed_checks.iter().cloned());
    caller.elided_accesses.extend(callee.elided_accesses.iter().cloned());
    caller.uncounted_handles.extend(callee.uncounted_handles.iter().map(|id| local(*id)));
    caller.for_iterators.extend(callee.for_iterators.iter().map(|id| local(*id)));
}

fn renumber_place(place: &mut Place, local: &impl Fn(LocalId) -> LocalId) {
    place.local = local(place.local);
    for projection in &mut place.projection {
        if let Projection::Index(index) = projection {
            *index = local(*index);
        }
    }
}

fn renumber_operand(operand: &mut Operand, local: &impl Fn(LocalId) -> LocalId) {
    if let Operand::Copy(place) | Operand::Move(place) = operand {
        renumber_place(place, local);
    }
}

fn renumber_stmt(stmt: &mut Stmt, local: &impl Fn(LocalId) -> LocalId) {
    match &mut stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            renumber_place(place, local);
            match rvalue {
                Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => {
                    renumber_operand(value, local)
                }
                Rvalue::Repeat { value, .. } => renumber_operand(value, local),
                Rvalue::BinaryOp { lhs, rhs, .. } => {
                    renumber_operand(lhs, local);
                    renumber_operand(rhs, local);
                }
                Rvalue::Aggregate { operands, .. } => {
                    for value in operands {
                        renumber_operand(value, local);
                    }
                }
                Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => renumber_place(place, local),
            }
        }
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            renumber_place(dest, local);
            renumber_place(overflow, local);
            renumber_operand(lhs, local);
            renumber_operand(rhs, local);
        }
        StmtKind::Drop { place, flag, .. } => {
            renumber_place(place, local);
            if let Some(flag) = flag {
                *flag = local(*flag);
            }
        }
        StmtKind::BeginAccess { place, .. }
        | StmtKind::BeginAccessTransfer { place, .. }
        | StmtKind::EndAccess { place, .. }
        | StmtKind::EndAccessTransfer { place, .. } => renumber_place(place, local),
        StmtKind::StorageLive(id) | StmtKind::StorageDead(id) => *id = local(*id),
        StmtKind::Nop => {}
    }
}

fn renumber_terminator(
    terminator: &mut Terminator,
    local: &impl Fn(LocalId) -> LocalId,
    target: &impl Fn(BasicBlockId) -> BasicBlockId,
) {
    match terminator {
        Terminator::Goto(next) => *next = target(*next),
        Terminator::SwitchInt { discr, targets, otherwise } => {
            renumber_operand(discr, local);
            for (_, next) in targets.iter_mut() {
                *next = target(*next);
            }
            *otherwise = target(*otherwise);
        }
        Terminator::Call { func, args, dest, next } => {
            if let FuncRef::Indirect { operand, .. } = func {
                renumber_operand(operand, local);
            }
            for arg in args.iter_mut() {
                renumber_operand(arg, local);
            }
            renumber_place(dest, local);
            *next = target(*next);
        }
        Terminator::Assert { cond, msg, next, .. } => {
            renumber_operand(cond, local);
            if let ember_mir::AssertKind::Bounds { len, index } = msg {
                renumber_operand(len, local);
                renumber_operand(index, local);
            }
            if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                renumber_operand(file, local);
                renumber_operand(line, local);
            }
            if let ember_mir::AssertKind::Panic { message } = msg {
                renumber_operand(message, local);
            }
            *next = target(*next);
        }
        Terminator::Return | Terminator::Unreachable => {}
    }
}
