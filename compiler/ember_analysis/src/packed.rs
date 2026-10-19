//! `[LAY-2]` — a field of a `@packed` value may sit at any address, so it is read and written by
//! value and a reference to it is `E2170`. Every reference MIR forms is an `Rvalue::Ref`: a
//! written `ref`, and the borrows a parameter mode makes (a `mut` argument, a borrowed argument
//! passed by address, a method's receiver), so the check is over those. A field aligned to one
//! byte is never unaligned and may be borrowed.

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{Body, Rvalue, StmtKind, unaligned_place};
use ember_types::TypeTable;
use std::collections::HashSet;

pub fn check_packed_references_all(bodies: &[Body], types: &TypeTable, sink: &mut Sink) -> usize {
    // One report per place in the source, however many generic instances repeat it.
    let mut reported = HashSet::new();
    for body in bodies {
        for stmt in body.blocks.iter().flat_map(|block| &block.stmts) {
            if let StmtKind::Assign { rvalue: Rvalue::Ref { place, .. }, .. } = &stmt.kind
                && unaligned_place(types, body, place)
                && reported.insert(stmt.span)
            {
                sink.emit(
                    Diagnostic::error(codes::E2170, stmt.span, "reference to a field of a packed struct")
                        .primary_label("this borrows a field that may be unaligned")
                        .help("read the field into a local first, and borrow the local")
                        .note("a `@packed` type has no padding, so its fields are read and written by value [LAY-2]"),
                );
            }
        }
    }
    reported.len()
}
