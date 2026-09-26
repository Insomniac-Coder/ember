//! `[EXC-7]` — warn when a mutable class-method access remains live across a
//! virtual or dynamic call in its own open-class hierarchy.
//!
//! A `mut self` method's access is its caller's write to every field of
//! `*self`, held for the whole call (`[EXC-15]`), which the MIR records as
//! `Body::mut_self`; explicit `BeginAccess`/`EndAccess` of `*self` inside the
//! body still end and restart it. Effect analysis is a later phase; until it
//! can prove a dispatch target access-free, the lint is conservative.

use std::collections::HashSet;

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{Body, FuncRef, LocalId, Operand, Place, Projection, Rvalue, StmtKind, Terminator};
use ember_types::{ClassId, ClassOpenness, TypeTable};

/// Emit the opt-in `[EXC-7]` lint for each reachable dynamic call that occurs
/// while its enclosing `mut self` access remains live.
pub fn lint_long_term_access_across_dynamic_calls_all(
    bodies: &[Body],
    types: &TypeTable,
    sink: &mut Sink,
) {
    for body in bodies {
        lint_body(body, types, sink);
    }
}

#[derive(Clone)]
struct LongTermAccess {
    place: Place,
    class: ClassId,
    span: ember_span::Span,
}

fn lint_body(body: &Body, types: &TypeTable, sink: &mut Sink) {
    let Some(access) = entry_class_access(body) else {
        return;
    };
    // `[EXC-7]` is about dispatch that may re-enter an *open* hierarchy. An
    // erased `ref dyn` call from a final class still has an indirect ABI, but
    // no unknown subclass can override its behavior, so it is not this lint's
    // long-term-access hazard.
    if types.class_def(access.class).openness == ClassOpenness::Final {
        return;
    }
    let mut pending = vec![(0usize, true)];
    let mut visited = HashSet::new();
    let mut reported = HashSet::new();

    while let Some((block_index, mut active)) = pending.pop() {
        if !visited.insert((block_index, active)) {
            continue;
        }
        let Some(block) = body.blocks.get(block_index) else {
            continue;
        };
        for statement in &block.stmts {
            match &statement.kind {
                StmtKind::BeginAccess { place, mutable }
                    if *mutable && *place == access.place =>
                {
                    active = true;
                }
                StmtKind::EndAccess { place, mutable }
                    if *mutable && *place == access.place =>
                {
                    active = false;
                }
                _ => {}
            }
        }

        if active
            && dynamic_call_in_hierarchy(body, types, &access, &block.terminator)
            && reported.insert(block_index)
        {
            let name = access_name(body, &access.place);
            sink.emit(
                Diagnostic::lint(
                    codes::L3013,
                    block.terminator_span,
                    format!("long-term access to `{name}` is live across this call"),
                )
                .primary_label("dynamic call happens here")
                .secondary(access.span, format!("long-term access to `{name}` begins here"))
                .help("end the access before the call, or move the call outside the mutable method")
                .note(
                    "a virtual or dynamic call can re-enter this open-class hierarchy [EXC-7]",
                ),
            );
        }

        for successor in successors(&block.terminator) {
            pending.push((successor, active));
        }
    }
}

/// A method-duration class access belongs only to a `mut self` method: its
/// caller writes every field of `*self` for the call (`[EXC-15]`). Short
/// intervals for call arguments do not qualify as long-term.
fn entry_class_access(body: &Body) -> Option<LongTermAccess> {
    let class = body.class_owner?;
    body.mut_self.then(|| LongTermAccess {
        place: Place { local: LocalId(1), projection: vec![Projection::Deref] },
        class,
        span: body.span,
    })
}

fn dynamic_call_in_hierarchy(
    body: &Body,
    types: &TypeTable,
    access: &LongTermAccess,
    terminator: &Terminator,
) -> bool {
    let Terminator::Call { func, args, .. } = terminator else {
        return false;
    };
    match func {
        FuncRef::Virtual { owner, .. } => same_hierarchy(types, access.class, *owner),
        FuncRef::Interface { .. } => args
            .first()
            .is_some_and(|receiver| operand_derives_from_access(body, receiver, &access.place)),
        _ => false,
    }
}

fn same_hierarchy(types: &TypeTable, left: ClassId, right: ClassId) -> bool {
    types.class_is_subclass_of(left, right) || types.class_is_subclass_of(right, left)
}

/// A `ref dyn I` carrier is materialized into a local before its table call.
/// Follow that local's `InterfaceUpcast`/copy origin back to the class receiver
/// rather than treating every dynamic call in a mutable method as a re-entry.
fn operand_derives_from_access(body: &Body, operand: &Operand, access: &Place) -> bool {
    let (Operand::Copy(place) | Operand::Move(place)) = operand else {
        return false;
    };
    place_derives_from_access(body, place, access, &mut HashSet::new())
}

fn place_derives_from_access(
    body: &Body,
    place: &Place,
    access: &Place,
    visited: &mut HashSet<LocalId>,
) -> bool {
    if *place == *access {
        return true;
    }
    if !place.projection.is_empty() || !visited.insert(place.local) {
        return false;
    }
    body.blocks.iter().any(|block| {
        block.stmts.iter().any(|statement| {
            let StmtKind::Assign { place: destination, rvalue } = &statement.kind else {
                return false;
            };
            if destination.local != place.local || !destination.projection.is_empty() {
                return false;
            }
            match rvalue {
                Rvalue::Use(operand) | Rvalue::Cast { operand, .. } => {
                    operand_derives_from_place(body, operand, access, visited)
                }
                Rvalue::Ref { place, .. } => {
                    place_derives_from_access(body, place, access, visited)
                }
                _ => false,
            }
        })
    })
}

fn operand_derives_from_place(
    body: &Body,
    operand: &Operand,
    access: &Place,
    visited: &mut HashSet<LocalId>,
) -> bool {
    let (Operand::Copy(place) | Operand::Move(place)) = operand else {
        return false;
    };
    place_derives_from_access(body, place, access, visited)
}

fn access_name(body: &Body, place: &Place) -> String {
    body.local(place.local)
        .name
        .clone()
        .unwrap_or_else(|| "this class object".to_string())
}

fn successors(terminator: &Terminator) -> Vec<usize> {
    match terminator {
        Terminator::Goto(target) => vec![target.0 as usize],
        Terminator::SwitchInt {
            targets, otherwise, ..
        } => targets
            .iter()
            .map(|(_, target)| target.0 as usize)
            .chain(std::iter::once(otherwise.0 as usize))
            .collect(),
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}
