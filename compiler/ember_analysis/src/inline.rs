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
    AggregateKind, BasicBlock, BasicBlockId, Body, FuncRef, LocalId, Operand, ParameterMode, Place, Projection, Rvalue,
    Stmt, StmtKind, Terminator,
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
                // `[TYP-9]` — a body keeps its own floating point: one of
                // another mode stays a call (`[CG-C-11]` compiles it apart).
                (callee != *caller
                    && bodies[callee].fp == bodies[*caller].fp
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

/// Whether a function's call boundary carries nothing but the call, and
/// the function may go once its one call is inlined.
fn inlinable(body: &Body, types: &TypeTable) -> bool {
    // A method (its first parameter `self`) may be named by an interface
    // table or drop glue, which find it by its type, not by a call.
    !(body.arg_count > 0 && body.locals.get(1).and_then(|l| l.name.as_deref()) == Some("self")) && copyable(body, types)
}

/// Whether a copy of a function's body may stand in for a call of it: the
/// call boundary carries nothing but the call.
fn copyable(body: &Body, types: &TypeTable) -> bool {
    // `[CG-C-3a]` — `@noinline` asks for a call, and `@cold` code is kept
    // out of the paths that call it.
    !body.inline.never
        && !body.inline.cold
        && body.abi.is_none()
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

/// `[CG-C-3]`, ADR-107 — for MSVC: a `for` loop's step through a standard
/// iterator (`next` of a type of std's) is inlined into the loop, and the
/// reference the step took to the loop's iterator is forwarded to the
/// iterator itself, so the C keeps the iterator as a local whose address is
/// never taken. MSVC keeps a struct whose address goes to a function in
/// memory even when it inlines the function: the separator of `split(",")`
/// was read from memory every turn and three loop values were spilled around
/// each `memchr` (1.12x the C loop; clang 0.95x on the same C). The step's
/// own body stays: interface tables and other loops still call it.
pub fn inline_loop_steps_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    let steps: HashMap<String, Body> = bodies
        .iter()
        .filter(|body| standard_step(body, types) && copyable(body, types) && !calls_itself(body))
        .map(|body| (body.symbol.clone(), body.clone()))
        .collect();
    if steps.is_empty() {
        return 0;
    }
    let mut inlined = 0;
    for body in bodies.iter_mut() {
        let here = inline_loop_steps(body, &steps);
        if here > 0 {
            thread_known_variants(body, types);
        }
        inlined += here;
    }
    inlined
}

/// A block that only asks which variant a local holds (`d = discriminant(x)`
/// then a switch on `d`), entered from a block that has just given `x` a
/// variant (`x = None`, `x = Some(...)`, or a copy of a local just given one),
/// is jumped over: the entering block goes straight to that variant's branch.
/// So with a step copied into its loop, the "no more items" the step builds
/// leaves the loop and an item runs the body, and the loop no longer tests
/// what the step gave back (ADR-107). `x` keeps the value, for the branch to
/// read. Not for a local whose address is taken, which a write through a
/// reference could change unseen, nor a `d` read anywhere but its switch.
fn thread_known_variants(body: &mut Body, types: &TypeTable) -> usize {
    let mut borrowed = vec![false; body.locals.len()];
    let mut reads = vec![0usize; body.locals.len()];
    for data in &body.blocks {
        for stmt in &data.stmts {
            if let StmtKind::Assign { rvalue: Rvalue::Ref { place, .. }, .. } = &stmt.kind {
                borrowed[place.local.0 as usize] = true;
            }
        }
        crate::loop_version::visit_places(data, &mut |place: &Place, written: bool, _: bool| {
            if !written {
                reads[place.local.0 as usize] += 1;
            }
        });
    }
    let mut threaded = 0;
    for test in 0..body.blocks.len() {
        let Some((x, d)) = variant_test(&body.blocks[test]) else { continue };
        if borrowed[x.0 as usize] || reads[d.0 as usize] != 1 {
            continue;
        }
        let TyKind::Enum(id) = *types.kind(body.locals[x.0 as usize].ty) else { continue };
        let Terminator::SwitchInt { targets, otherwise, .. } = body.blocks[test].terminator.clone() else { continue };
        for entering in 0..body.blocks.len() {
            if entering == test || !matches!(body.blocks[entering].terminator, Terminator::Goto(to) if to.0 as usize == test) {
                continue;
            }
            let Some(variant) = known_variant(&body.blocks[entering], x) else { continue };
            let value = types.enum_def(id).variants[variant].discriminant;
            let branch = targets.iter().find(|(v, _)| *v == value).map_or(otherwise, |(_, to)| *to);
            body.blocks[entering].terminator = Terminator::Goto(branch);
            threaded += 1;
        }
    }
    threaded
}

/// `d = discriminant(x)` and nothing else, then a switch on `d`: `(x, d)`.
fn variant_test(data: &BasicBlock) -> Option<(LocalId, LocalId)> {
    let mut found = None;
    for stmt in &data.stmts {
        match &stmt.kind {
            StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            StmtKind::Assign { place, rvalue: Rvalue::Discriminant(of) }
                if found.is_none() && place.projection.is_empty() && of.projection.is_empty() =>
            {
                found = Some((of.local, place.local));
            }
            _ => return None,
        }
    }
    let (x, d) = found?;
    match &data.terminator {
        Terminator::SwitchInt { discr: Operand::Copy(on) | Operand::Move(on), .. } if on.local == d && on.projection.is_empty() => {
            Some((x, d))
        }
        _ => None,
    }
}

/// The variant `x` holds at the end of `data`, when the block gives it one:
/// an enum value built whole, or a whole copy of a local the block gave one.
/// Any other write to a local forgets what was known of it.
fn known_variant(data: &BasicBlock, x: LocalId) -> Option<usize> {
    let mut known: HashMap<LocalId, usize> = HashMap::new();
    for stmt in &data.stmts {
        match &stmt.kind {
            StmtKind::Assign { place, rvalue } if place.projection.is_empty() => {
                let variant = match rvalue {
                    Rvalue::Aggregate { kind: AggregateKind::Enum(_, variant), .. } => Some(*variant),
                    Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) if from.projection.is_empty() => {
                        known.get(&from.local).copied()
                    }
                    _ => None,
                };
                match variant {
                    Some(variant) => known.insert(place.local, variant),
                    None => known.remove(&place.local),
                };
            }
            StmtKind::StorageLive(_) | StmtKind::Nop => {}
            StmtKind::StorageDead(local) => {
                known.remove(local);
            }
            _ => {
                let mut written = Vec::new();
                crate::loop_version::visit_places(
                    &BasicBlock { stmts: vec![stmt.clone()], terminator: Terminator::Unreachable, terminator_span: stmt.span },
                    &mut |place: &Place, is_written: bool, _: bool| {
                        if is_written {
                            written.push(place.local);
                        }
                    },
                );
                for local in written {
                    known.remove(&local);
                }
            }
        }
    }
    known.get(&x).copied()
}

/// `next` of a type of std's: a method of that name whose receiver is a
/// mutable reference to a std struct or enum.
fn standard_step(body: &Body, types: &TypeTable) -> bool {
    if body.name != "next" || body.arg_count != 1 {
        return false;
    }
    let TyKind::Ref { mutable: true, inner } = types.kind(body.locals[1].ty) else { return false };
    let name = match types.kind(*inner) {
        TyKind::Struct(id) => types.struct_def(*id).name,
        TyKind::Enum(id) => types.enum_def(*id).name,
        _ => return false,
    };
    name.as_str().starts_with("std.")
}

fn inline_loop_steps(body: &mut Body, steps: &HashMap<String, Body>) -> usize {
    let mut inlined = 0;
    // A step inlined appends blocks and keeps every block's number, so the
    // blocks there before are each looked at once.
    let before = body.blocks.len();
    for block in 0..before {
        let Terminator::Call { func: FuncRef::Direct { symbol, .. }, args, .. } = &body.blocks[block].terminator else { continue };
        let Some(step) = steps.get(symbol) else { continue };
        let [Operand::Copy(reference) | Operand::Move(reference)] = args.as_slice() else { continue };
        if !reference.projection.is_empty() {
            continue;
        }
        let reference = reference.local;
        let Some(iterator) = loop_iterator(body, reference) else { continue };
        let first_local = body.locals.len() as u32;
        let first_block = body.blocks.len();
        inline_call(body, block, step);
        let parameter = LocalId(first_local + 1);
        forward(body, block, first_block, reference, parameter, iterator);
        inlined += 1;
    }
    inlined
}

/// The loop iterator a reference was made from: `r = &mut it`, the one
/// value `r` is ever given, `it` one of the body's `for` iterators.
fn loop_iterator(body: &Body, reference: LocalId) -> Option<LocalId> {
    let mut found = None;
    for data in &body.blocks {
        for stmt in &data.stmts {
            if let StmtKind::Assign { place, rvalue } = &stmt.kind
                && place.local == reference
            {
                let Rvalue::Ref { place: target, mutable: true } = rvalue else { return None };
                if !place.projection.is_empty() || !target.projection.is_empty() || found.is_some_and(|seen| seen != target.local) {
                    return None;
                }
                found = Some(target.local);
            }
        }
    }
    found.filter(|iterator| body.for_iterators.contains(iterator))
}

/// Every place the inlined step reaches through its parameter (`*p…`) is
/// now reached in the iterator itself (`it…`), when the parameter is used
/// for nothing but that; the copy `p = r` and, once nothing else reads
/// `r`, `r = &mut it` go, so nothing takes the iterator's address.
fn forward(body: &mut Body, call_block: usize, first_block: usize, reference: LocalId, parameter: LocalId, iterator: LocalId) {
    let mut only_through = true;
    for data in &body.blocks[first_block..] {
        crate::loop_version::visit_places(data, &mut |place: &Place, _: bool, _: bool| {
            if place.local == parameter && place.projection.first() != Some(&Projection::Deref) {
                only_through = false;
            }
        });
    }
    if !only_through {
        return;
    }
    for data in &mut body.blocks[first_block..] {
        crate::loop_version::rewrite_places(data, &|place: &mut Place| {
            if place.local == parameter && place.projection.first() == Some(&Projection::Deref) {
                place.local = iterator;
                place.projection.remove(0);
            }
        });
    }
    body.blocks[call_block].stmts.retain(|stmt| {
        !matches!(&stmt.kind, StmtKind::Assign { place, rvalue: Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) }
            if place.local == parameter && place.projection.is_empty() && from.local == reference && from.projection.is_empty())
    });
    if !reads(body, reference) {
        for data in &mut body.blocks {
            data.stmts.retain(|stmt| {
                !matches!(&stmt.kind, StmtKind::Assign { place, rvalue: Rvalue::Ref { .. } } if place.local == reference && place.projection.is_empty())
            });
        }
    }
}

/// Whether any statement or terminator of the body reads `local`.
fn reads(body: &Body, local: LocalId) -> bool {
    let mut read = vec![false; body.locals.len()];
    for data in &body.blocks {
        for stmt in &data.stmts {
            crate::strength_reduce::stmt_reads(stmt, &mut read);
        }
        crate::strength_reduce::terminator_reads(&data.terminator, &mut read);
    }
    read[local.0 as usize]
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
