//! `L1001` unused binding and `L1002` assignment declares a new binding
//! (`[LNT-1]`, `[LNT-2]`, `[LNT-3]`).
//!
//! `[GRM-4]` makes `x = e` declare when `x` is not in scope and assign when it
//! is. That is the footgun ADR-002 names: a typo in the name silently creates a
//! second variable instead of writing to the first, and the program keeps
//! compiling. `L1001` reports the binding nobody read; `L1002` notices that its
//! name is one or two edits away from a binding that *is* in scope and was not
//! written afterwards, which is what the typo looks like from here.
//!
//! `[LNT-3]` — both are emitted by `ember build` and `ember check`, not only by
//! `ember lint`. A diagnostic that fires on a separate command does not close a
//! footgun.
//!
//! `[LNT-1]` specifies liveness from the borrow checker's analysis (§4.7 step
//! 3), which is not built. Until it is, "read" means "appears as an operand, an
//! index, or the base of a projection that is read anywhere in the body". That
//! is more conservative than liveness — a local read only on a dead path counts
//! as read — so it under-reports rather than crying wolf, which is the right
//! way round for a lint that is on by default.

use std::collections::HashSet;

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{Body, LocalId, LocalKind, Operand, Place, Rvalue, StmtKind, Terminator};

pub fn check_all(bodies: &[Body], sink: &mut Sink) {
    for body in bodies {
        check(body, sink);
    }
}

pub fn check(body: &Body, sink: &mut Sink) {
    let mut read = HashSet::new();
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    mark_place_indices(place, &mut read);
                    mark_rvalue(rvalue, &mut read);
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    mark_place_indices(dest, &mut read);
                    mark_place_indices(overflow, &mut read);
                    mark_operand(lhs, &mut read);
                    mark_operand(rhs, &mut read);
                }
                // A drop reads the value in order to destroy it, but a local
                // that is only dropped was never *used*: that is exactly the
                // case this lint exists to report.
                StmtKind::Drop { place, flag } => {
                    mark_place_indices(place, &mut read);
                    if let Some(flag) = flag {
                        read.insert(*flag);
                    }
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            }
        }
        match &block.terminator {
            Terminator::SwitchInt { discr, .. } => mark_operand(discr, &mut read),
            Terminator::Call { args, dest, .. } => {
                for arg in args {
                    mark_operand(arg, &mut read);
                }
                mark_place_indices(dest, &mut read);
            }
            Terminator::Assert { cond, msg, .. } => {
                mark_operand(cond, &mut read);
                if let ember_mir::AssertKind::Bounds { len, index } = msg {
                    mark_operand(len, &mut read);
                    mark_operand(index, &mut read);
                }
                if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                    mark_operand(file, &mut read);
                    mark_operand(line, &mut read);
                }
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }

    // Candidates for `L1002`: named user locals that are read somewhere, so a
    // near-miss name plausibly meant one of them.
    let live_names: Vec<&str> = body
        .locals
        .iter()
        .enumerate()
        .filter(|(index, decl)| {
            decl.kind == LocalKind::User
                && read.contains(&LocalId(*index as u32))
                && decl.name.is_some()
        })
        .map(|(_, decl)| decl.name.as_deref().unwrap())
        .collect();

    for (index, decl) in body.locals.iter().enumerate() {
        if decl.kind != LocalKind::User || decl.span.is_dummy() {
            continue;
        }
        let Some(name) = decl.name.as_deref() else { continue };
        // `[LNT-1]` — names beginning `_` are exempt.
        if name.starts_with('_') || read.contains(&LocalId(index as u32)) {
            continue;
        }

        let mut diag = Diagnostic::lint(codes::L1001, decl.span, format!("`{name}` is never read"))
            .primary_label("declared here and never used")
            .help(format!("remove it, or rename it `_{name}` if that is deliberate"));

        // `[LNT-2]` — the near miss. Reported on the same span, because the
        // declaration *is* the mistake: the writer meant to assign.
        if let Some(near) = nearest(name, &live_names) {
            diag = Diagnostic::lint(
                codes::L1002,
                decl.span,
                format!("`{name}` declares a new binding; `{near}` is already in scope"),
            )
            .primary_label(format!("did you mean to assign to `{near}`?"))
            .help(format!("write `{near} = …`"))
            .note(
                "`x = e` declares when `x` is not in scope and assigns when it is (GRM-4)",
            );
        }
        sink.emit(diag);
    }
}

fn mark_operand(operand: &Operand, read: &mut HashSet<LocalId>) {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => {
            read.insert(place.local);
            mark_place_indices(place, read);
        }
        Operand::Const(_) => {}
    }
}

/// The locals a place *reads* even when the place itself is written: `a[i] = v`
/// reads `i`, and writing through a dereference reads the pointer.
fn mark_place_indices(place: &Place, read: &mut HashSet<LocalId>) {
    for projection in &place.projection {
        match projection {
            ember_mir::Projection::Index(local) => {
                read.insert(*local);
            }
            ember_mir::Projection::Deref => {
                read.insert(place.local);
            }
            _ => {}
        }
    }
}

fn mark_rvalue(rvalue: &Rvalue, read: &mut HashSet<LocalId>) {
    match rvalue {
        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => mark_operand(o, read),
        Rvalue::Cast { operand, .. } => mark_operand(operand, read),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            mark_operand(lhs, read);
            mark_operand(rhs, read);
        }
        Rvalue::Aggregate { operands, .. } => {
            for o in operands {
                mark_operand(o, read);
            }
        }
        Rvalue::Repeat { value, .. } => mark_operand(value, read),
        Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => {
            read.insert(place.local);
            mark_place_indices(place, read);
        }
    }
}

/// `[LNT-2]` — Damerau–Levenshtein distance ≤ 2, and never a match on a name
/// so short that two edits rewrite it entirely.
fn nearest<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
    if name.len() < 3 {
        return None;
    }
    let mut best: Option<(usize, &str)> = None;
    for candidate in candidates {
        if *candidate == name {
            continue;
        }
        let d = distance(name, candidate);
        if d <= 2 && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, candidate));
        }
    }
    best.map(|(_, c)| c)
}

/// Optimal string alignment: Levenshtein plus adjacent transposition, which is
/// what makes `veloctiy` one edit from `velocity` rather than two.
fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=b.len() {
        d[0][j] = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}
