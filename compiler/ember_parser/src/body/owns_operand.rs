//! `[GRM-47]` — `owns` is never an operand: an operator reads its operands in place, so `owns i * 7`
//! is `(owns i) * 7`, which ends `i` before the multiplication (`E0119`).

use ember_ast::{Expr, ExprKind};
use ember_diag::{Diagnostic, codes};
use ember_span::Span;

use crate::Parser;

/// An `owns` expression met as an operand, waiting for the call or the `=` it stands in.
pub(crate) struct OwnsOperand {
    /// The `owns` expression: from the word to the end of what it hands over.
    owns: Span,
    /// What follows the word.
    inner: Span,
    op: &'static str,
    /// Where the operator expression ends, and whether the `owns` expression is its left operand.
    whole_end: u32,
    leftmost: bool,
}

pub(crate) enum OwnsPlace {
    Argument { call_start: u32 },
    Assigned { stmt_start: u32, value_start: u32 },
    Elsewhere,
}

fn noun(op: &str) -> &'static str {
    match op {
        "+" => "addition",
        "-" => "subtraction",
        "*" => "multiplication",
        "/" | "//" => "division",
        "%" => "remainder",
        "**" => "power",
        _ => "operation",
    }
}

impl Parser<'_> {
    /// Notes the operand when it is an `owns` expression, with or without brackets of its own, and
    /// leaves the value it names in its place, so that the checker reports nothing more of it.
    pub(crate) fn note_owns_operand(&mut self, operand: &mut Expr, op: &'static str, leftmost: bool) {
        let mut e = operand;
        while matches!(e.kind, ExprKind::Paren(_)) {
            let ExprKind::Paren(inner) = &mut e.kind else { unreachable!() };
            e = inner;
        }
        let ExprKind::Owned(inner) = &mut e.kind else { return };
        let (owns, inner_span) = (e.span, inner.span);
        let whole_end = self.prev_span().end;
        self.owns_operands.push(OwnsOperand { owns, inner: inner_span, op, whole_end, leftmost });
        let ExprKind::Owned(inner) = std::mem::replace(&mut e.kind, ExprKind::Error) else { unreachable!() };
        *e = *inner;
    }

    fn text(&self, start: u32, end: u32) -> &str {
        self.src.get(start as usize..end as usize).unwrap_or("")
    }

    /// Reports what was noted since `from`, with the line to write for the place it stands in.
    pub(crate) fn flush_owns_operands(&mut self, from: usize, place: OwnsPlace) -> Vec<Span> {
        let noted: Vec<OwnsOperand> = self.owns_operands.drain(from..).collect();
        let spans: Vec<Span> = noted.iter().map(|o| o.owns).collect();
        for o in noted {
            let name = self.text(o.inner.start, o.inner.end).trim().to_string();
            let what = noun(o.op);
            let head = format!("`owns` here is an operand of `{}`, which only reads `{name}`", o.op);
            // The operator expression without the word: what the line says when nothing is handed over.
            let without = |p: &Self, a: u32, b: u32| format!("{}{}", p.text(a, o.owns.start), p.text(o.inner.start, b));
            let diagnostic = match place {
                OwnsPlace::Argument { call_start } if o.leftmost => {
                    // The word moves in front of the whole product, which goes in brackets.
                    let rest = self.text(o.whole_end, self.src.len() as u32);
                    let close = rest.find(')').map_or(o.whole_end, |i| o.whole_end + i as u32 + 1);
                    let line = format!(
                        "{}owns ({}){}",
                        self.text(call_start, o.owns.start),
                        self.text(o.inner.start, o.whole_end),
                        self.text(o.whole_end, close)
                    );
                    Diagnostic::error(codes::E0119, o.owns, head)
                        .primary_label(format!("`{name}` would be handed over before the {what}, and the product is a new value"))
                        .help(format!("write the product in brackets: `{line}`"))
                }
                OwnsPlace::Assigned { stmt_start, value_start } => {
                    let line = format!("{}{}", self.text(stmt_start, value_start), without(self, value_start, o.whole_end));
                    Diagnostic::error(codes::E0119, o.owns, head)
                        .primary_label(format!("`{name}` would end before the {what}, and `=` hands the product over with no word"))
                        .help(format!("write `{}`", line.trim()))
                }
                _ => {
                    let line = without(self, o.owns.start, o.whole_end);
                    Diagnostic::error(codes::E0119, o.owns, head)
                        .primary_label(format!("`{name}` would end before the {what}"))
                        .help(format!("an operator reads its operands in place: remove `owns` and write `{}`", line.trim()))
                }
            };
            self.report(diagnostic);
        }
        spans
    }
}
