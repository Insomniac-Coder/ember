//! `[EFF-12]`, `[EFF-13]` — a `@static_safe` function keeps no dynamic
//! aliasing check (`E4030`, shape S1): neither a class exclusivity check
//! (`BeginAccess`, hoisted or not) nor a `RefCell` borrow check. Bounds,
//! arithmetic and stale-handle checks are allowed.
//!
//! The verdict is taken once, after the profile-independent proofs (`[EXC-3]`'s
//! elisions, `[EXC-8]`/`[EXC-10]`'s hoisting, the never-firing checks) and
//! before inlining, so a check is the function's own and the answer is the
//! same in every profile (`[EFF-15]`).
//!
//! `[DIA-11]` — each report names the check's reason (`[EFF-11]`) and suggests what that reason
//! leaves the programmer: a per-access check the analysis did not prove away
//! (`not_proven_by_analysis`), a restructuring; a loop's one check, which is what exclusivity on a
//! shared object is (`inherent_to_mechanism`), removing the contract or changing the data
//! structure, never restructuring code that cannot be improved; a `RefCell`'s
//! (`requested_by_type`), changing the type.

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{AssertKind, Body, StmtKind, Terminator};

/// What was found, and what fixes it.
enum Site {
    Access,
    LoopAccess,
    RefCell,
}

pub fn check_static_safe_all(bodies: &[Body], sink: &mut Sink) -> usize {
    let mut reported = 0;
    for body in bodies.iter().filter(|body| body.static_safe) {
        let mut sites = Vec::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            for (index, stmt) in block.stmts.iter().enumerate() {
                if matches!(stmt.kind, StmtKind::BeginAccess { .. } | StmtKind::BeginAccessTransfer { .. }) {
                    let hoisted = body.hoisted_accesses.iter().any(|record| {
                        record.preheader.0 as usize == block_index && record.preheader_statement == index
                    });
                    sites.push((stmt.span, if hoisted { Site::LoopAccess } else { Site::Access }));
                }
            }
            if let Terminator::Assert { msg: AssertKind::RefCellBorrow { .. }, span, .. } = &block.terminator {
                sites.push((*span, Site::RefCell));
            }
        }
        for (span, site) in sites {
            let (what, reason, help) = match site {
                Site::Access => (
                    "an exclusivity check on a class object",
                    "`not_proven_by_analysis`: the access may be exclusive, but that was not proved",
                    "hoist the handle into a local the code does not reassign, so the access is proved when it \
                     compiles, or use value types or a query yielding `ref mut`",
                ),
                Site::LoopAccess => (
                    "an exclusivity check on a class object, once for the loop",
                    "`inherent_to_mechanism`: one check for the loop is what exclusivity on a shared object is",
                    "no restructuring of this code removes it: remove `@static_safe`, or change the data \
                     structure to value types or a query yielding `ref mut`",
                ),
                Site::RefCell => (
                    "a `RefCell` borrow check",
                    "`requested_by_type`: a `RefCell` checks every borrow at run time",
                    "change the type: pass the value itself, or a `ref mut` to it, so the access is proved when \
                     it compiles",
                ),
            };
            sink.emit(
                Diagnostic::error(
                    codes::E4030,
                    span,
                    format!("`@static_safe` function `{}` keeps {what} here", body.name),
                )
                .primary_label("checked at run time")
                .note(format!("the check's reason is {reason} [EFF-11]"))
                .help(help)
                .note(
                    "`@static_safe` allows bounds, arithmetic and stale-handle checks, but no check of who else \
                     holds the data (EFF-12, EFF-13)",
                ),
            );
            reported += 1;
        }
    }
    reported
}
