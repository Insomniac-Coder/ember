//! ADR-141 — a short list or string a function makes, fills and drops itself
//! starts in a 64-byte buffer of the function's own frame, as C++'s short
//! strings do: it moves to the heap only when it outgrows the buffer, and its
//! drop frees only a heap block. `m[f"key{i}"]` then allocates nothing.
//!
//! A local `L` of type `Array[T]` or `String` gets a buffer when:
//! * every definition of `L` is an `Array.new()` / `String.new()` call;
//! * every other use is a shared borrow, an element read, write or borrow, a
//!   drop with no flag, `len(L)`, or `M = &mut L` where `M` is set once and
//!   only ever the list an in-place built-in changes (a push, an insert, a
//!   reserve, a removal, a reorder, or a format of a built-in value). Those
//!   grow the list only through the runtime's reallocation, which tells a
//!   frame buffer from a heap block; nothing else can free, move or replace
//!   the list;
//! * `T` needs no drop, and its size is 1 to 64 bytes and alignment at most 8;
//! * no call can come back to the function while it runs (a cycle of direct
//!   calls, or a call the compiler cannot follow), so a recursion's stack
//!   stays as it was.
//!
//! At most four lists a function.

use std::collections::{HashMap, HashSet};

use ember_mir::{
    AssertKind, Body, Builtin, Const, FuncRef, LocalId, LocalKind, Operand, Place, Projection, Rvalue, StackBuffer,
    StmtKind, Terminator,
};
use ember_types::{Ty, TyKind, TypeTable};

const BUFFER_BYTES: u64 = 64;
const MOST_PER_BODY: usize = 4;

/// Give every such list in `bodies` its buffer (`Body::stack_buffers`);
/// returns how many were given.
pub fn give_stack_buffers_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    let open = may_come_back(bodies);
    let mut given = 0;
    for (index, body) in bodies.iter_mut().enumerate() {
        if open[index] || body.is_extern_declaration || body.is_abstract {
            continue;
        }
        let lists: Vec<StackBuffer> = (body.arg_count as u32 + 1..body.locals.len() as u32)
            .map(LocalId)
            .filter_map(|local| buffer_for(body, types, local))
            .take(MOST_PER_BODY)
            .collect();
        given += lists.len();
        body.stack_buffers = lists;
    }
    given
}

/// Whether each body may be called again while it runs: it is on a cycle of
/// direct calls, or it reaches a call the compiler cannot follow (through a
/// value, a vtable or an interface, into foreign code, or a built-in given a
/// function), which may call anything.
fn may_come_back(bodies: &[Body]) -> Vec<bool> {
    let at: HashMap<&str, usize> = bodies.iter().enumerate().map(|(i, body)| (body.symbol.as_str(), i)).collect();
    let mut calls: Vec<Vec<usize>> = vec![Vec::new(); bodies.len()];
    let mut open = vec![false; bodies.len()];
    for (i, body) in bodies.iter().enumerate() {
        for block in &body.blocks {
            let Terminator::Call { func, args, .. } = &block.terminator else { continue };
            match func {
                FuncRef::Direct { symbol, .. } => match at.get(symbol.as_str()) {
                    Some(&callee) if !bodies[callee].is_extern_declaration => calls[i].push(callee),
                    _ => open[i] = true,
                },
                FuncRef::Builtin { .. } => {
                    if args.iter().any(|arg| matches!(arg, Operand::Const(Const::Fn(_)))) {
                        open[i] = true;
                    }
                }
                _ => open[i] = true,
            }
        }
    }
    // Reaching a call the compiler cannot follow, through direct calls.
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..bodies.len() {
            if !open[i] && calls[i].iter().any(|&callee| open[callee]) {
                open[i] = true;
                changed = true;
            }
        }
    }
    // On a cycle of direct calls: the body reaches itself.
    for i in 0..bodies.len() {
        if open[i] {
            continue;
        }
        let mut seen = HashSet::new();
        let mut work = calls[i].clone();
        while let Some(next) = work.pop() {
            if next == i {
                open[i] = true;
                break;
            }
            if seen.insert(next) {
                work.extend(&calls[next]);
            }
        }
    }
    open
}

/// How a statement or terminator names a place.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Use {
    Read,
    Write,
    Borrow { mutable: bool },
    Drop { flagged: bool },
    Other,
}

fn operand_uses<'a>(operand: &'a Operand, out: &mut Vec<(&'a Place, Use)>) {
    if let Operand::Copy(place) | Operand::Move(place) = operand {
        out.push((place, Use::Read));
    }
}

/// Every place a statement names, with how.
fn stmt_uses(stmt: &StmtKind) -> Vec<(&Place, Use)> {
    let mut out = Vec::new();
    match stmt {
        StmtKind::Assign { place, rvalue } => {
            out.push((place, Use::Write));
            match rvalue {
                Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => {
                    operand_uses(value, &mut out)
                }
                Rvalue::BinaryOp { lhs, rhs, .. } => {
                    operand_uses(lhs, &mut out);
                    operand_uses(rhs, &mut out);
                }
                Rvalue::Aggregate { operands, .. } => operands.iter().for_each(|value| operand_uses(value, &mut out)),
                Rvalue::Repeat { value, .. } => operand_uses(value, &mut out),
                Rvalue::Discriminant(place) => out.push((place, Use::Read)),
                Rvalue::Ref { place, mutable } => out.push((place, Use::Borrow { mutable: *mutable })),
            }
        }
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            out.push((dest, Use::Write));
            out.push((overflow, Use::Write));
            operand_uses(lhs, &mut out);
            operand_uses(rhs, &mut out);
        }
        StmtKind::Drop { place, flag, .. } => out.push((place, Use::Drop { flagged: flag.is_some() })),
        StmtKind::BeginAccess { place, .. }
        | StmtKind::BeginAccessTransfer { place, .. }
        | StmtKind::EndAccess { place, .. }
        | StmtKind::EndAccessTransfer { place, .. } => out.push((place, Use::Other)),
        StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
    }
    out
}

/// Every place a terminator names, with how.
fn terminator_uses(terminator: &Terminator) -> Vec<(&Place, Use)> {
    let mut out = Vec::new();
    match terminator {
        Terminator::SwitchInt { discr, .. } => operand_uses(discr, &mut out),
        Terminator::Call { args, dest, .. } => {
            args.iter().for_each(|arg| operand_uses(arg, &mut out));
            out.push((dest, Use::Write));
        }
        Terminator::Assert { cond, msg, .. } => {
            operand_uses(cond, &mut out);
            if let AssertKind::Bounds { len, index } = msg {
                operand_uses(len, &mut out);
                operand_uses(index, &mut out);
            }
        }
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
    }
    out
}

fn whole(place: &Place, local: LocalId) -> bool {
    place.local == local && place.projection.is_empty()
}

/// Whether `place` names `local` itself or something inside it.
fn names(place: &Place, local: LocalId) -> bool {
    place.local == local || place.projection.iter().any(|p| matches!(p, Projection::Index(index) if *index == local))
}

/// An element of `local`: `local[i]`, and anything below it.
fn element(place: &Place, local: LocalId) -> bool {
    place.local == local && matches!(place.projection.first(), Some(Projection::Index(_) | Projection::ConstIndex(_)))
}

/// The buffer `local` gets, if it qualifies.
fn buffer_for(body: &Body, types: &TypeTable, local: LocalId) -> Option<StackBuffer> {
    let decl = body.local(local);
    if !matches!(decl.kind, LocalKind::User | LocalKind::Temp) {
        return None;
    }
    let TyKind::Vec { elem, .. } = *types.kind(decl.ty) else { return None };
    let layout = types.layout(elem);
    if types.needs_drop(elem) || layout.size == 0 || layout.size > BUFFER_BYTES || layout.align > 8 {
        return None;
    }
    // Whether one use of `local` (not its making) is allowed.
    let allowed = |place: &Place, how: Use| match how {
        Use::Read | Use::Write | Use::Borrow { mutable: true } => element(place, local),
        Use::Borrow { mutable: false } => true,
        Use::Drop { flagged } => !flagged && whole(place, local),
        Use::Other => false,
    };
    let mut lenders = HashSet::new();
    let mut made = false;
    for block in &body.blocks {
        for stmt in &block.stmts {
            // `M = &mut L`, which lends `L` to its growth.
            if let StmtKind::Assign { place, rvalue: Rvalue::Ref { place: lent, mutable: true } } = &stmt.kind
                && whole(lent, local)
            {
                if place.projection.is_empty() && place.local != local {
                    lenders.insert(place.local);
                    continue;
                }
                return None;
            }
            if stmt_uses(&stmt.kind).into_iter().any(|(place, how)| names(place, local) && !allowed(place, how)) {
                return None;
            }
        }
        if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayNew | Builtin::StringNew, .. }, dest, .. } =
            &block.terminator
            && whole(dest, local)
        {
            made = true;
            continue;
        }
        // `len(L)` reads a copy of the header.
        if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayLen | Builtin::StringLen, .. }, args, dest, .. } =
            &block.terminator
            && matches!(args.first(), Some(Operand::Copy(place)) if whole(place, local))
            && !names(dest, local)
        {
            continue;
        }
        if terminator_uses(&block.terminator).into_iter().any(|(place, how)| names(place, local) && !allowed(place, how)) {
            return None;
        }
    }
    if !made || lenders.iter().any(|&lender| !only_grows(body, types, lender)) {
        return None;
    }
    Some(StackBuffer { local, capacity: BUFFER_BYTES / layout.size })
}

/// Whether `lender` (`= &mut L`) is set once and only ever the list a growth
/// call writes to.
fn only_grows(body: &Body, types: &TypeTable, lender: LocalId) -> bool {
    let mut sets = 0;
    for block in &body.blocks {
        for stmt in &block.stmts {
            if let StmtKind::Assign { place, rvalue: Rvalue::Ref { mutable: true, .. } } = &stmt.kind
                && whole(place, lender)
            {
                sets += 1;
                continue;
            }
            if stmt_uses(&stmt.kind).into_iter().any(|(place, _)| names(place, lender)) {
                return false;
            }
        }
        if let Terminator::Call { func: FuncRef::Builtin { which, arg_ty }, args, dest, .. } = &block.terminator
            && grows(which, *arg_ty, types)
            && matches!(args.first(), Some(Operand::Copy(place) | Operand::Move(place)) if whole(place, lender))
        {
            let rest = args[1..].iter().any(|arg| matches!(arg, Operand::Copy(p) | Operand::Move(p) if names(p, lender)));
            if rest || names(dest, lender) {
                return false;
            }
            continue;
        }
        if terminator_uses(&block.terminator).into_iter().any(|(place, _)| names(place, lender)) {
            return false;
        }
    }
    sets == 1
}

/// A built-in that changes the list it is lent in place, growing it only
/// through the runtime's reallocation: a push, an insert, a reserve, a
/// removal, a reorder, or a format of a value the runtime formats itself (a
/// user type's formatting runs user code).
fn grows(which: &Builtin, arg_ty: Ty, types: &TypeTable) -> bool {
    match which {
        Builtin::ArrayReserve
        | Builtin::ArrayReserveHint { .. }
        | Builtin::ArrayPush
        | Builtin::ArrayInsert
        | Builtin::ArrayPop { .. }
        | Builtin::ArrayRemove
        | Builtin::ArraySwapRemove
        | Builtin::ArrayTruncate
        | Builtin::ArrayClear
        | Builtin::ArraySwap
        | Builtin::ArrayReverse
        | Builtin::StringPush
        | Builtin::StringPushChar
        | Builtin::StringInsert
        | Builtin::StringRemove
        | Builtin::StringTruncate => true,
        Builtin::Format | Builtin::FormatWith(_) => matches!(
            types.kind(arg_ty),
            TyKind::Bool
                | TyKind::Char
                | TyKind::Int(_)
                | TyKind::Uint(_)
                | TyKind::Float(_)
                | TyKind::Str
                | TyKind::Void
                | TyKind::Vec { text: true, .. }
        ),
        _ => false,
    }
}
