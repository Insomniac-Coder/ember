//! Conservative dynamic-exclusivity loop hoisting (`[EXC-8]`–`[EXC-14]`).
//!
//! This pass recognizes only the canonical counted-loop shape emitted by MIR
//! lowering.  Its receiver is an unprojected class local; each iteration may
//! only prepare a field reference and make one direct call through that
//! reference. The direct call must also carry a verified summary constrained
//! to that distinct field reference. Every shape outside that narrow proof
//! boundary remains an ordinary `[EXC-1]` dynamic access.

use std::collections::{BTreeMap, HashMap, HashSet};

use ember_mir::{
    BasicBlock, BasicBlockId, Body, CallableAccessSummary, FuncRef, HoistedAccess,
    HoistedAccessProof, LocalId, LocalKind, Operand, Place, Rvalue, Stmt, StmtKind, Terminator,
};
use ember_types::{TyKind, TypeTable};

/// Hoist every independently-proved canonical counted-loop access in `bodies`.
/// The checker has already established the source program's NLL regions; this
/// pass preserves their bracket semantics by retaining the same begin/end
/// statements and changing only the CFG that reaches them.
pub fn hoist_loop_accesses_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    let contracts = bodies
        .iter()
        .filter_map(|body| {
            body.callable_regions
                .as_ref()
                .filter(|metadata| metadata.fingerprint_is_valid())
                .map(|metadata| (body.symbol.clone(), metadata.access.clone()))
        })
        .collect();
    bodies
        .iter_mut()
        .map(|body| hoist_loop_accesses(body, types, &contracts))
        .sum()
}

fn hoist_loop_accesses(
    body: &mut Body,
    types: &TypeTable,
    contracts: &HashMap<String, CallableAccessSummary>,
) -> usize {
    let original_block_count = body.blocks.len();
    let mut count = 0;
    for header in 0..original_block_count {
        let Some(candidate) = candidate(body, types, contracts, header) else {
            continue;
        };
        apply(body, candidate);
        count += 1;
    }
    // `[EXC-10]` — then each held access moves out to the loops around it.
    while let Some((record, outer)) =
        (0..body.hoisted_accesses.len()).find_map(|record| outer_loop(body, types, record).map(|outer| (record, outer)))
    {
        lift(body, record, outer);
        count += 1;
    }
    count
}

/// The loop around a hoisted access that may hold it instead (`[EXC-10]`),
/// its blocks in order and the one block its every way out reaches.
struct Outer {
    header: usize,
    blocks: Vec<usize>,
    exit: usize,
}

/// A loop held access: copying the outer loop doubles it, so a large one stays.
const MAX_OUTER_BLOCKS: usize = 256;

/// `[EXC-10]`, `[EXC-9]` — the smallest loop strictly around access `record`'s
/// bracket that makes no other access, drops nothing, runs no Ember code and
/// never names the receiver. Its handle stays the one object: no `&mut` of it
/// is taken anywhere but for the bracket's own proven calls (a `mut self`
/// call cannot re-point `self`, ODR-072). The bracket's own inside was
/// proved by `[EXC-8]` (or by an earlier step of this one).
fn outer_loop(body: &Body, types: &TypeTable, record: usize) -> Option<Outer> {
    let access = &body.hoisted_accesses[record];
    let (preheader, postheader) = (access.preheader.0 as usize, access.postheader.0 as usize);
    let receiver = access.place.local;
    if !access.place.projection.is_empty() {
        return None;
    }
    // The bracket: what runs from its begin to its end.
    let mut bracket = HashSet::from([preheader]);
    let mut work = vec![preheader];
    while let Some(block) = work.pop() {
        if block == postheader {
            continue;
        }
        for next in crate::long_access_lint::successors(&body.blocks[block].terminator) {
            if bracket.insert(next) {
                work.push(next);
            }
        }
    }
    if !bracket.contains(&postheader) {
        return None;
    }
    let order = crate::range_facts::reverse_postorder(body);
    let headers = crate::range_facts::loop_headers(body, &order);
    let (header, blocks) = crate::range_facts::natural_loops(body, &order, &headers)
        .into_iter()
        .filter(|(header, blocks)| !bracket.contains(header) && bracket.iter().all(|block| blocks.contains(block)))
        .min_by_key(|(_, blocks)| blocks.len())?;
    if blocks.len() > MAX_OUTER_BLOCKS {
        return None;
    }
    let mut exits = HashSet::new();
    for &block in &blocks {
        for next in crate::long_access_lint::successors(&body.blocks[block].terminator) {
            if !blocks.contains(&next) {
                exits.insert(next);
            }
        }
    }
    let [exit] = exits.into_iter().collect::<Vec<_>>()[..] else { return None };
    // A reference to the whole handle is the bracket's call setup only: never
    // copied anywhere, and in the loop passed only to the bracket's calls.
    // A call after the loop (another nest's) cannot re-point it inside.
    let mut references = HashSet::new();
    for data in &body.blocks {
        for stmt in &data.stmts {
            if let StmtKind::Assign { place, rvalue: Rvalue::Ref { place: borrowed, mutable: true } } = &stmt.kind
                && borrowed.local == receiver
                && borrowed.projection.is_empty()
            {
                references.insert(place.local);
            }
        }
    }
    for (index, data) in body.blocks.iter().enumerate() {
        let in_bracket = bracket.contains(&index);
        for stmt in &data.stmts {
            let defines = matches!(&stmt.kind, StmtKind::Assign { place, rvalue: Rvalue::Ref { place: borrowed, .. } }
                if references.contains(&place.local) && borrowed.local == receiver);
            if !defines && references.iter().any(|&reference| statement_mentions_local(stmt, reference)) {
                return None;
            }
        }
        if blocks.contains(&index)
            && references.iter().any(|&reference| terminator_mentions_local(&data.terminator, reference))
            && !(in_bracket && matches!(&data.terminator, Terminator::Call { func: FuncRef::Direct { .. }, .. }))
        {
            return None;
        }
    }
    // Outside the bracket, the loop only counts: pure statements, built-ins
    // that run no Ember code, branches; never the receiver.
    // The bracket's call setup (`&mut` of the receiver, above) may come
    // before its begin, in the loop's turn.
    let sets_up = |stmt: &Stmt| matches!(&stmt.kind,
        StmtKind::Assign { place, rvalue: Rvalue::Ref { place: borrowed, .. } }
            if references.contains(&place.local) && borrowed.local == receiver && borrowed.projection.is_empty());
    for &block in blocks.iter().filter(|block| !bracket.contains(block)) {
        let data = &body.blocks[block];
        if data.stmts.iter().any(|stmt| !matches!(&stmt.kind,
                StmtKind::Assign { .. } | StmtKind::CheckedBinaryOp { .. }
                | StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop)
            || (statement_mentions_local(stmt, receiver) && !sets_up(stmt))
            || matches!(&stmt.kind, StmtKind::Assign { place, .. } if !place.projection.is_empty()))
            || terminator_mentions_local(&data.terminator, receiver)
        {
            return None;
        }
        if let Terminator::Call { func, args, .. } = &data.terminator {
            let FuncRef::Builtin { which, .. } = func else { return None };
            if !crate::check_hoisting::runs_no_ember_code(body, types, which, args) {
                return None;
            }
        }
    }
    // Inside it, only its own begin and end.
    for &block in &bracket {
        for (index, stmt) in body.blocks[block].stmts.iter().enumerate() {
            let own = (block == preheader && index == access.preheader_statement)
                || (block == postheader
                    && matches!(&stmt.kind, StmtKind::EndAccess { place, mutable }
                        if *place == access.place && *mutable == access.mutable));
            if !own && matches!(&stmt.kind, StmtKind::BeginAccess { .. } | StmtKind::EndAccess { .. }
                | StmtKind::BeginAccessTransfer { .. } | StmtKind::EndAccessTransfer { .. })
            {
                return None;
            }
        }
    }
    let mut blocks: Vec<usize> = blocks.into_iter().collect();
    blocks.sort_unstable();
    Some(Outer { header, blocks, exit })
}

/// `[EXC-10]` — the outer loop holds the access. Its turns run as written
/// until the first one whose inner loop begins the access; from that begin
/// a copy of the loop without the bracket's begin and end runs, and its way
/// out ends the access. A loop whose inner loops never run checks nothing,
/// as before, and the first check fails where it failed before.
fn lift(body: &mut Body, record: usize, outer: Outer) {
    let access = body.hoisted_accesses[record].clone();
    let (preheader, postheader) = (access.preheader.0 as usize, access.postheader.0 as usize);
    let first = body.blocks.len();
    let copies: BTreeMap<usize, usize> = outer.blocks.iter().enumerate().map(|(i, &b)| (b, first + i)).collect();
    let end_block = first + outer.blocks.len();
    let id = |b: usize| BasicBlockId(b as u32);
    let held = |target: BasicBlockId| {
        let target = target.0 as usize;
        id(if target == outer.exit { end_block } else { copies.get(&target).copied().unwrap_or(target) })
    };
    let end = body.blocks[postheader]
        .stmts
        .iter()
        .find(|stmt| matches!(&stmt.kind, StmtKind::EndAccess { place, mutable }
            if *place == access.place && *mutable == access.mutable))
        .cloned()
        .expect("a hoisted access ends in its postheader");
    let mut copied = Vec::new();
    for &block in &outer.blocks {
        let mut data = body.blocks[block].clone();
        if block == preheader {
            data.stmts.remove(access.preheader_statement);
        }
        if block == postheader {
            data.stmts.retain(|stmt| !matches!(&stmt.kind, StmtKind::EndAccess { place, mutable }
                if *place == access.place && *mutable == access.mutable && stmt.span == end.span));
        }
        crate::loop_version::retarget(&mut data.terminator, held);
        copied.push(data);
    }
    copied.push(BasicBlock {
        stmts: vec![end],
        terminator: Terminator::Goto(id(outer.exit)),
        terminator_span: body.blocks[postheader].terminator_span,
        call_span: None,
    });
    // After its begin, the original goes on in the copy that holds it.
    crate::loop_version::retarget(&mut body.blocks[preheader].terminator, |target| {
        copies.get(&(target.0 as usize)).map_or(target, |&copy| id(copy))
    });
    let loop_span = body.blocks[outer.header].terminator_span;
    body.blocks.extend(copied);
    let access = &mut body.hoisted_accesses[record];
    access.postheader = id(end_block);
    access.loop_span = loop_span;
    access.proof = HoistedAccessProof::InnerLoopReused;
}

/// A closed access contained in the exact CFG shape `lower_for_range` emits.
/// Keeping all anchors explicit makes the subsequent rewrite auditable and
/// ensures an unfamiliar loop shape fails closed rather than approximating a
/// source-level region.
struct Candidate {
    header: usize,
    body: usize,
    end: usize,
    step: usize,
    /// D-525 — an inclusive loop's test for its last turn, before the step:
    /// it leaves the loop too, so through the postheader.
    last: Option<usize>,
    exit: BasicBlockId,
    begin_index: usize,
    place: Place,
    mutable: bool,
    span: ember_span::Span,
    loop_span: ember_span::Span,
}

fn candidate(
    body: &Body,
    types: &TypeTable,
    contracts: &HashMap<String, CallableAccessSummary>,
    header: usize,
) -> Option<Candidate> {
    let (body_block, exit) = counted_loop_targets(body.blocks.get(header)?)?;
    let body_index = body_block.0 as usize;
    let exit_index = exit.0 as usize;
    if body_index >= body.blocks.len() || exit_index >= body.blocks.len() {
        return None;
    }

    let loop_body = body.blocks.get(body_index)?;
    let (func, args, next) = match &loop_body.terminator {
        Terminator::Call {
            func, args, next, ..
        } => (func, args, *next),
        _ => return None,
    };
    // A virtual, interface, indirect, built-in, or unknown call is precisely
    // the case `[EXC-9](d)` requires us to leave dynamically checked.
    if !matches!(func, FuncRef::Direct { .. }) {
        return None;
    }
    let end = next.0 as usize;
    let end_block = body.blocks.get(end)?;
    let Terminator::Goto(step_block) = end_block.terminator else {
        return None;
    };
    let mut step = step_block.0 as usize;
    // D-525 — an inclusive loop tests for its last turn before it steps; the
    // test leaves where the header's does.
    let mut last = None;
    if let Terminator::SwitchInt { targets, otherwise, .. } = &body.blocks.get(step)?.terminator
        && targets.len() == 1
        && targets[0].0 == 0
        && *otherwise == exit
    {
        last = Some(step);
        step = targets[0].1.0 as usize;
    }
    if step >= body.blocks.len()
        || !matches!(body.blocks.get(step)?.terminator, Terminator::Goto(target) if target.0 as usize == header)
    {
        return None;
    }

    let predecessors = predecessors(body);
    // A canonical counted loop is entered from its setup block and its one
    // increment block; alternative entries or back edges would need a more
    // general proof before an interval can safely span the loop.
    if predecessors.get(header)?.len() != 2
        || predecessors.get(body_index)?.as_slice() != [header]
        || predecessors.get(end)?.as_slice() != [body_index]
        || last.is_some_and(|test| predecessors.get(step).is_none_or(|from| from.as_slice() != [test]))
    {
        return None;
    }

    let (begin_index, begin) =
        loop_body
            .stmts
            .iter()
            .enumerate()
            .find_map(|(index, statement)| match &statement.kind {
                StmtKind::BeginAccess { place, mutable } if *mutable => {
                    Some((index, (place.clone(), *mutable, statement.span)))
                }
                _ => None,
            })?;
    let (place, mutable, span) = begin;
    if !place.projection.is_empty()
        || !matches!(types.kind(body.local(place.local).ty), TyKind::Class(_))
        || loop_body
            .stmts
            .iter()
            .skip(begin_index + 1)
            .next()
            .is_some()
    {
        return None;
    }
    // Either a call on a class held in one of the receiver's fields, whose
    // summary shows it touches only that field, or a `mut self` call on the
    // receiver itself (`[EXC-15]`): it held the receiver's write access for
    // each call, and between calls only the counter runs. `self` cannot be
    // re-pointed inside it (ODR-072), so the object stays the same.
    let (setup_refs, kept) = match setup_references(body, types, &loop_body.stmts[..begin_index], place.local) {
        Some((setup_refs, kept))
            if direct_call_is_modelled_for_setup_fields(func, args, place.local, &setup_refs, contracts) =>
        {
            (setup_refs, kept)
        }
        _ => (vec![mut_self_call_on_root(body, &loop_body.stmts[..begin_index], func, args, place.local)?], Vec::new()),
    };
    // The matching end opens the post-call block, and all that may follow it
    // there is the release of the receiver copies ODR-065 retains for the call
    // (`kept`). This both proves that the access cannot escape the source
    // iteration and gives the rewrite a single, total continuation path. The
    // modelled callee (below) touches no field of the object, so each copy's
    // release finds the field still holding the handle and runs no code.
    if end_block.stmts.is_empty()
        || !matches!(
            &end_block.stmts[0].kind,
            StmtKind::EndAccess { place: end_place, mutable: end_mutable }
                if end_place == &place && *end_mutable == mutable
        )
        || !end_block.stmts[1..].iter().all(|statement| match &statement.kind {
            StmtKind::Drop { place: dropped, .. } => dropped.projection.is_empty() && kept.contains(&dropped.local),
            // D-342 — the end of a statement temporary's storage runs no code.
            StmtKind::StorageDead(local) => {
                kept.contains(local) || setup_refs.contains(local) || body.local(*local).kind == LocalKind::Temp
            }
            _ => false,
        })
    {
        return None;
    }
    // The loop condition and step cannot read, overwrite, publish, or
    // invalidate the object identity.  This is the local part of
    // `[EXC-9](a)` and `(c)`.
    if block_mentions_local(body.blocks.get(header)?, place.local)
        || block_mentions_local(body.blocks.get(step)?, place.local)
        || last.is_some_and(|test| body.blocks.get(test).is_none_or(|test| block_mentions_local(test, place.local)))
    {
        return None;
    }

    Some(Candidate {
        header,
        body: body_index,
        end,
        step,
        last,
        exit,
        begin_index,
        place,
        mutable,
        span,
        loop_span: body.blocks[header].terminator_span,
    })
}

/// Return the temporary locals formed only as field references into the stable
/// receiver. These instructions are compiler setup, not a source-level escape
/// or a call boundary. Any richer setup is left per-access until it has its
/// own proof rule.
fn setup_references(
    body: &Body,
    types: &TypeTable,
    statements: &[Stmt],
    root: LocalId,
) -> Option<(Vec<LocalId>, Vec<LocalId>)> {
    let mut references = Vec::new();
    // ODR-065 — a receiver read from the object's own field is a retained
    // copy of it, borrowed for the call.
    let mut kept = Vec::new();
    for statement in statements {
        match &statement.kind {
            StmtKind::StorageLive(_) => {}
            // D-342 — the end of an earlier statement temporary's storage.
            StmtKind::StorageDead(local) if body.local(*local).kind == LocalKind::Temp => {}
            StmtKind::Assign { place: copy, rvalue: Rvalue::Use(Operand::Copy(field)) }
                if copy.projection.is_empty()
                    && field.local == root
                    && !field.projection.is_empty()
                    && matches!(types.kind(body.local(copy.local).ty), TyKind::Class(_)) =>
            {
                kept.push(copy.local)
            }
            StmtKind::Assign {
                place: destination,
                rvalue: Rvalue::Ref { place: borrowed, mutable: true },
            } if destination.projection.is_empty()
                && borrowed.projection.is_empty()
                && kept.contains(&borrowed.local)
                && is_distinct_class_reference(body, types, root, destination.local) =>
            {
                references.push(destination.local)
            }
            StmtKind::Assign {
                place: destination,
                rvalue:
                    Rvalue::Ref {
                        place: borrowed,
                        mutable: true,
                    },
            } if destination.projection.is_empty()
                && borrowed.local == root
                && !borrowed.projection.is_empty()
                && is_distinct_class_reference(body, types, root, destination.local) =>
            {
                references.push(destination.local)
            }
            _ => return None,
        }
    }
    Some((references, kept))
}

/// `[EXC-15]` — the loop body only borrows the receiver for a direct `mut self`
/// call on it: `t = &mut root` and `call(t, …)`, where no other argument
/// names the receiver or `t`. The reference is the one the call takes.
fn mut_self_call_on_root(
    body: &Body,
    statements: &[Stmt],
    func: &FuncRef,
    args: &[Operand],
    root: LocalId,
) -> Option<LocalId> {
    if !matches!(func, FuncRef::Direct { .. }) {
        return None;
    }
    let mut reference = None;
    for statement in statements {
        match &statement.kind {
            StmtKind::StorageLive(_) => {}
            StmtKind::StorageDead(local) if body.local(*local).kind == LocalKind::Temp => {}
            StmtKind::Assign { place: destination, rvalue: Rvalue::Ref { place: borrowed, mutable: true } }
                if destination.projection.is_empty() && *borrowed == Place::local(root) && reference.is_none() =>
            {
                reference = Some(destination.local);
            }
            _ => return None,
        }
    }
    let reference = reference?;
    let (Operand::Copy(receiver) | Operand::Move(receiver)) = args.first()? else { return None };
    if *receiver != Place::local(reference) {
        return None;
    }
    args[1..]
        .iter()
        .all(|argument| !operand_mentions_local(argument, root) && !operand_mentions_local(argument, reference))
        .then_some(reference)
}

/// A direct callee must have a verified finite summary, and each summarized
/// argument must be an exact field-reference capability. This rules out both
/// opaque call effects and a reference that could identify the protected
/// object itself, satisfying `[EXC-9](d)` for this conservative first slice.
fn direct_call_is_modelled_for_setup_fields(
    func: &FuncRef,
    args: &[Operand],
    root: LocalId,
    setup_refs: &[LocalId],
    contracts: &HashMap<String, CallableAccessSummary>,
) -> bool {
    let FuncRef::Direct { symbol, .. } = func else {
        return false;
    };
    if !args.iter().all(|argument| match argument {
        Operand::Copy(place) | Operand::Move(place) if place.local == root => false,
        Operand::Copy(place) | Operand::Move(place) if setup_refs.contains(&place.local) => {
            place.projection.is_empty()
        }
        Operand::Copy(place) | Operand::Move(place) => !place_mentions_local(place, root),
        Operand::Const(_) => true,
    }) {
        return false;
    }
    let Some(CallableAccessSummary::Fields(accesses)) = contracts.get(symbol) else {
        return false;
    };
    accesses.iter().all(|access| {
        args.get(access.argument).is_some_and(|argument| {
            matches!(
                argument,
                Operand::Copy(place) | Operand::Move(place)
                    if place.projection.is_empty() && setup_refs.contains(&place.local)
            )
        })
    })
}

/// A field can point back to its owning class (or a related base/derived
/// object), so a mere projected borrow does not prove non-aliasing. Restrict
/// this first rule to nominally unrelated class references; richer alias proof
/// stays dynamic until a later rule can establish it.
fn is_distinct_class_reference(
    body: &Body,
    types: &TypeTable,
    root: LocalId,
    reference: LocalId,
) -> bool {
    let TyKind::Class(root_class) = types.kind(body.local(root).ty) else {
        return false;
    };
    let TyKind::Ref { inner, .. } = types.kind(body.local(reference).ty) else {
        return false;
    };
    let TyKind::Class(referenced_class) = types.kind(*inner) else {
        return false;
    };
    !types.class_is_subclass_of(*referenced_class, *root_class)
        && !types.class_is_subclass_of(*root_class, *referenced_class)
}

fn block_mentions_local(block: &BasicBlock, local: LocalId) -> bool {
    block
        .stmts
        .iter()
        .any(|statement| statement_mentions_local(statement, local))
        || terminator_mentions_local(&block.terminator, local)
}

fn statement_mentions_local(statement: &Stmt, local: LocalId) -> bool {
    match &statement.kind {
        StmtKind::Assign { place, rvalue } => {
            place_mentions_local(place, local) || rvalue_mentions_local(rvalue, local)
        }
        StmtKind::BeginAccess { place, .. }
        | StmtKind::BeginAccessTransfer { place, .. }
        | StmtKind::EndAccess { place, .. }
        | StmtKind::EndAccessTransfer { place, .. } | StmtKind::LendAccess { place, .. }
        | StmtKind::Drop { place, .. } => place_mentions_local(place, local),
        StmtKind::CheckedBinaryOp {
            dest,
            overflow,
            lhs,
            rhs,
            ..
        } => {
            place_mentions_local(dest, local)
                || place_mentions_local(overflow, local)
                || operand_mentions_local(lhs, local)
                || operand_mentions_local(rhs, local)
        }
        StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => false,
    }
}

fn rvalue_mentions_local(rvalue: &Rvalue, local: LocalId) -> bool {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => {
            operand_mentions_local(operand, local)
        }
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            operand_mentions_local(lhs, local) || operand_mentions_local(rhs, local)
        }
        Rvalue::Aggregate { operands, .. } => operands
            .iter()
            .any(|operand| operand_mentions_local(operand, local)),
        Rvalue::Repeat { value, .. } => operand_mentions_local(value, local),
        Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => {
            place_mentions_local(place, local)
        }
    }
}

fn terminator_mentions_local(terminator: &Terminator, local: LocalId) -> bool {
    match terminator {
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => false,
        Terminator::SwitchInt { discr, .. } => operand_mentions_local(discr, local),
        Terminator::Call {
            func, args, dest, ..
        } => {
            place_mentions_local(dest, local)
                || args
                    .iter()
                    .any(|argument| operand_mentions_local(argument, local))
                || matches!(func, FuncRef::Indirect { operand, .. } if operand_mentions_local(operand, local))
        }
        Terminator::Assert { cond, msg, .. } => {
            operand_mentions_local(cond, local)
                || match msg {
                    ember_mir::AssertKind::Bounds { len, index } => {
                        operand_mentions_local(len, local) || operand_mentions_local(index, local)
                    }
                    ember_mir::AssertKind::RefCellBorrow { file, line } => {
                        operand_mentions_local(file, local) || operand_mentions_local(line, local)
                    }
                    ember_mir::AssertKind::Overflow(_)
                    | ember_mir::AssertKind::DivisionByZero
                    | ember_mir::AssertKind::SignedDivisionOverflow
                    | ember_mir::AssertKind::ShiftTooLarge
                    | ember_mir::AssertKind::Downcast => false,
                    ember_mir::AssertKind::Panic { message } => operand_mentions_local(message, local),
                }
        }
    }
}

fn operand_mentions_local(operand: &Operand, local: LocalId) -> bool {
    matches!(operand, Operand::Copy(place) | Operand::Move(place) if place_mentions_local(place, local))
}

fn place_mentions_local(place: &Place, local: LocalId) -> bool {
    place.local == local
        || place
            .projection
            .iter()
            .any(|projection| matches!(projection, ember_mir::Projection::Index(index) if *index == local))
}

fn counted_loop_targets(block: &BasicBlock) -> Option<(BasicBlockId, BasicBlockId)> {
    match &block.terminator {
        Terminator::SwitchInt {
            targets, otherwise, ..
        } if targets.len() == 1 && targets[0].0 == 0 => Some((*otherwise, targets[0].1)),
        _ => None,
    }
}

fn predecessors(body: &Body) -> Vec<Vec<usize>> {
    let mut result = vec![Vec::new(); body.blocks.len()];
    for (index, block) in body.blocks.iter().enumerate() {
        for successor in successors(&block.terminator) {
            if let Some(entries) = result.get_mut(successor.0 as usize) {
                entries.push(index);
            }
        }
    }
    result
}

fn successors(terminator: &Terminator) -> Vec<BasicBlockId> {
    match terminator {
        Terminator::Goto(target) => vec![*target],
        Terminator::SwitchInt {
            targets, otherwise, ..
        } => targets
            .iter()
            .map(|(_, target)| *target)
            .chain(std::iter::once(*otherwise))
            .collect(),
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![*next],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn apply(body: &mut Body, candidate: Candidate) {
    let original_body = body.blocks[candidate.body].clone();
    let original_header = body.blocks[candidate.header].clone();
    let begin = original_body.stmts[candidate.begin_index].clone();
    let end = body.blocks[candidate.end].stmts[0].clone();
    let prelude = original_body.stmts[..candidate.begin_index].to_vec();

    // The first successful loop test reaches this short preheader after the
    // usual compiler-created field-reference setup. It opens the exact access
    // that used to start in the body, then every subsequent iteration joins
    // the shared call block under the same internal MIR interval.
    let begin_block = BasicBlockId(body.blocks.len() as u32);
    let call_block = BasicBlockId(begin_block.0 + 1);
    let regular_prelude = BasicBlockId(begin_block.0 + 2);
    let steady_header = BasicBlockId(begin_block.0 + 3);
    let postheader = BasicBlockId(begin_block.0 + 4);

    body.blocks[candidate.body] = BasicBlock {
        stmts: prelude.clone(),
        terminator: Terminator::Goto(begin_block),
        terminator_span: original_body.terminator_span,
        call_span: None,
    };
    // The access ends at the loop's exit now; the copies' releases stay.
    body.blocks[candidate.end].stmts.remove(0);
    body.blocks[candidate.step].terminator = Terminator::Goto(steady_header);
    // D-525 — the last turn's test leaves through the access's end too.
    if let Some(test) = candidate.last
        && let Terminator::SwitchInt { otherwise, .. } = &mut body.blocks[test].terminator
    {
        *otherwise = postheader;
    }

    body.blocks.push(BasicBlock {
        stmts: vec![begin],
        terminator: Terminator::Goto(call_block),
        terminator_span: original_body.terminator_span,
        call_span: None,
    });
    body.blocks.push(BasicBlock {
        stmts: Vec::new(),
        terminator: original_body.terminator,
        terminator_span: original_body.terminator_span,
        call_span: None,
    });
    body.blocks.push(BasicBlock {
        stmts: prelude,
        terminator: Terminator::Goto(call_block),
        terminator_span: original_body.terminator_span,
        call_span: None,
    });
    body.blocks.push(BasicBlock {
        stmts: original_header.stmts,
        terminator: redirect_loop_targets(original_header.terminator, regular_prelude, postheader),
        terminator_span: original_header.terminator_span,
        call_span: None,
    });
    body.blocks.push(BasicBlock {
        stmts: vec![end],
        terminator: Terminator::Goto(candidate.exit),
        terminator_span: body.blocks[candidate.end].terminator_span,
        call_span: None,
    });

    body.hoisted_accesses.push(HoistedAccess {
        span: candidate.span,
        loop_span: candidate.loop_span,
        preheader: begin_block,
        preheader_statement: 0,
        postheader,
        place: candidate.place,
        mutable: candidate.mutable,
        proof: HoistedAccessProof::StableReceiverDirectCall,
    });
}

fn redirect_loop_targets(
    terminator: Terminator,
    body: BasicBlockId,
    exit: BasicBlockId,
) -> Terminator {
    match terminator {
        Terminator::SwitchInt {
            discr,
            mut targets,
            otherwise: _,
        } => {
            // `candidate` only admits one `0` target, so this preserves the
            // original loop condition while changing only the two CFG edges.
            targets[0].1 = exit;
            Terminator::SwitchInt {
                discr,
                targets,
                otherwise: body,
            }
        }
        _ => unreachable!("only canonical counted-loop headers are redirected"),
    }
}
