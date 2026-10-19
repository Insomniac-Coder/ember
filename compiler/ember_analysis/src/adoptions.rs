//! `[FFI-36b]` — one live foreign object adopted twice through one local in one function is
//! `E5068`: an `adopt` of a local that an earlier `adopt` of the same local reaches, on some path,
//! with nothing written to the local between. The runtime's record of owned addresses catches every
//! other second owner (`adopt` in a helper called twice, a copy in another local).
//!
//! Only two distinct calls count: one call reached again through a loop's back edge is left to the
//! run-time check, since the loop may end first.

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{Body, Builtin, FuncRef, LocalId, Operand, Rvalue, StmtKind, Terminator};
use ember_span::Span;
use std::collections::BTreeMap;

/// Each local adopted on some path to here, with the `adopt` calls that adopted it.
type State = BTreeMap<LocalId, Vec<Span>>;

pub fn check_adoptions_all(bodies: &[Body], sink: &mut Sink) -> usize {
    bodies.iter().map(|body| check_adoptions(body, sink)).sum()
}

fn check_adoptions(body: &Body, sink: &mut Sink) -> usize {
    let adopts = |terminator: &Terminator| matches!(terminator,
        Terminator::Call { func: FuncRef::Builtin { which: Builtin::ForeignAdopt, .. }, .. });
    if body.blocks.iter().filter(|block| adopts(&block.terminator)).count() < 2 {
        return 0;
    }
    let mut entry: Vec<Option<State>> = vec![None; body.blocks.len()];
    entry[0] = Some(State::new());
    let mut found: Vec<(Span, Span)> = Vec::new();
    let mut worklist = vec![0usize];
    while let Some(index) = worklist.pop() {
        let Some(mut state) = entry[index].clone() else { continue };
        let block = &body.blocks[index];
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    if place.projection.is_empty() {
                        state.remove(&place.local);
                    }
                    // A `mut` borrow or address lets a callee write the local.
                    if let Rvalue::Ref { place, mutable: true } = rvalue {
                        state.remove(&place.local);
                    }
                }
                _ => {}
            }
        }
        if let Terminator::Call { func, args, dest, .. } = &block.terminator {
            let adopted = match (func, args.as_slice()) {
                (FuncRef::Builtin { which: Builtin::ForeignAdopt, .. }, [Operand::Copy(place) | Operand::Move(place)])
                    if place.projection.is_empty() => Some(place.local),
                _ => None,
            };
            if dest.projection.is_empty() {
                state.remove(&dest.local);
            }
            if let Some(local) = adopted {
                let here = block.terminator_span;
                let earlier = state.entry(local).or_default();
                if let Some(&first) = earlier.iter().find(|&&span| span != here) {
                    if !found.contains(&(here, first)) {
                        found.push((here, first));
                    }
                }
                if !earlier.contains(&here) {
                    earlier.push(here);
                }
            }
        }
        let successors: Vec<usize> = match &block.terminator {
            Terminator::Goto(next) | Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
            Terminator::SwitchInt { targets, otherwise, .. } => {
                targets.iter().map(|(_, target)| target.0 as usize).chain([otherwise.0 as usize]).collect()
            }
            Terminator::Return | Terminator::Unreachable => Vec::new(),
        };
        for successor in successors {
            let merged = match &entry[successor] {
                Some(existing) => {
                    let mut joined = existing.clone();
                    for (local, spans) in &state {
                        let into = joined.entry(*local).or_default();
                        for span in spans {
                            if !into.contains(span) {
                                into.push(*span);
                            }
                        }
                    }
                    if joined == *existing {
                        continue;
                    }
                    joined
                }
                None => state.clone(),
            };
            entry[successor] = Some(merged);
            worklist.push(successor);
        }
    }
    for &(here, first) in &found {
        sink.emit(
            Diagnostic::error(codes::E5068, here, "this foreign object is already adopted")
                .primary_label("adopted a second time here")
                .secondary(first, "first adopted here")
                .note("two `ForeignBox`es would each free it; the second `adopt` panics at run time [FFI-36b]")
                .help("adopt it once and pass the box, or `adopt` a different object"),
        );
    }
    found.len()
}
