//! `[DIA-7]` — the ownership and borrow diagnostic classifier.
//!
//! "Every ownership or borrow error MUST be classified into one of the shapes
//! in §XIX.6.1 and MUST emit that shape's required `help`. An error the
//! classifier cannot place emits the generic explanation and is recorded in
//! `target/<profile>/unclassified-borrow-errors.log`; CI fails if the
//! conformance suite produces any unclassified borrow error."
//!
//! The rule exists because "an unexplained rejection is the single largest
//! usability cost of static aliasing rules". A borrow checker that says *no*
//! without saying *what to write instead* is the failure mode, and this is the
//! mechanism that makes it impossible to add one by accident.
//!
//! Why it lives here rather than in a later diagnostics pass: `[DIA-7]`'s
//! classifier needs the borrow checker's own loan and region tables, so a pass
//! that runs afterwards over rendered text cannot do it. The shape is chosen
//! where the error is raised, and this module is what makes it impossible to
//! raise one without choosing.

use std::fmt;

use crate::codes;

/// One of §XIX.6.1's classified shapes. Every ownership or borrow diagnostic
/// carries one.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Shape {
    /// use after move
    O1,
    /// move out of a container
    O2,
    /// move in a loop
    O3,
    /// partial move then whole use
    O4,
    /// closure moves out a capture
    O5,
    /// borrow of a moved or uninitialised place
    O6,
    /// explicit `drop` call
    O7,
    /// leaked `@must_drop` value
    O8,
    /// `self` escapes its own drop
    O9,
    /// two mutable indices
    B1,
    /// mutate while iterating
    B2,
    /// shared and mutable overlap
    B3,
    /// aliased mutation of a value type
    B4,
    /// self-referential struct
    B5,
    /// returned reference not derived from a parameter
    B6,
    /// borrow outlives its source
    B7,
    /// method takes all of `self`
    B8,
    /// closure outlives its captures
    B9,
    /// `mut` argument is not a mutable place
    B10,
    /// disjointness not provable
    B11,
    /// view stored in a place that outlives it
    B12,
    /// two independent regions in one view struct
    B13,
    /// static exclusivity conflict
    X1,
    /// `@static_safe` violated
    S1,
    /// arena lifetime
    A1,
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Shape {
    /// The one-line description §XIX.6.1 gives, for the unclassified log and
    /// for `ember explain`.
    pub fn trigger(self) -> &'static str {
        match self {
            Shape::O1 => "a place is read after being moved",
            Shape::O2 => "a move out of an index, a span element, or a field of a `Drop` type",
            Shape::O3 => "a value declared outside a loop is moved inside it",
            Shape::O4 => "a struct is used as a whole after one field moved out",
            Shape::O5 => "a closure body moves out a captured non-`Copy` value",
            Shape::O6 => "a borrow of a place that is not initialised on every path",
            Shape::O7 => "`drop` called explicitly",
            Shape::O8 => "a scope guard moved out, forgotten, or stored where it may cycle",
            Shape::O9 => "a `drop` body publishes a handle to the object being destroyed",
            Shape::B1 => "two mutable borrows through computed indices",
            Shape::B2 => "a container mutated while it is being iterated",
            Shape::B3 => "a shared borrow live across a mutating access",
            Shape::B4 => "the same place needs two writers",
            Shape::B5 => "a struct field would borrow another field of the same struct",
            Shape::B6 => "elision cannot tie the return's region to an input",
            Shape::B7 => "a loan is live past the storage of what it borrows",
            Shape::B8 => "disjoint-field access defeated by a method call",
            Shape::B9 => "a non-`owned` closure escapes",
            Shape::B10 => "a temporary or immutable binding passed to a `mut` parameter",
            Shape::B11 => "two views whose ranges cannot be related",
            Shape::B12 => "a view stored where it outlives its source",
            Shape::B13 => "a view struct needing two independent regions",
            Shape::X1 => "two overlapping long-term accesses through one handle local",
            Shape::S1 => "a `@static_safe` function carries a dynamic aliasing check",
            Shape::A1 => "an arena view outliving its arena",
        }
    }
}

/// `[DIA-7a]` — "The table below maps every error code in `E3000–E3499` to the
/// shape whose `help` it MUST emit. A code absent from this table MUST NOT be
/// emitted."
///
/// The seven `E3021`–`E3027` rows are errata ERR-021: §XIX.6.1 specifies those
/// shapes down to their required `help`, and v0.5 gives them no code at all,
/// which left `[BRW-1]` itself unreportable.
pub fn shape_for(code: codes::Code) -> Option<Shape> {
    let shape = match code.number {
        3010 | 3011 | 3012 | 3013 => Shape::O2,
        3014 | 3015 => Shape::O8,
        3016 => Shape::O9,
        3020 => Shape::B2,
        3021 => Shape::B3,
        3022 => Shape::B1,
        3023 => Shape::B4,
        3024 => Shape::B5,
        3025 => Shape::B8,
        3026 => Shape::B9,
        3027 => Shape::B10,
        3030 => Shape::O5,
        3040 => Shape::O1,
        3041 => Shape::O3,
        3042 => Shape::O4,
        3050 => Shape::O6,
        3060 => Shape::B7,
        3061 | 3090 | 3096 => Shape::A1,
        3062 => Shape::B6,
        3063 => Shape::B12,
        3064 => Shape::B13,
        3070 => Shape::O7,
        3080 => Shape::X1,
        3095 => Shape::B11,
        4030 => Shape::S1,
        _ => return None,
    };
    Some(shape)
}

/// Codes in the ownership range that are **not** ownership or borrow errors,
/// so the classifier has nothing to say about them.
///
/// `[DIA-7a]` sits directly under `[DIA-7]`, which scopes the classifier to
/// "every ownership or borrow error"; read together, `[DIA-7a]`'s "every error
/// code in `E3000–E3499`" means every such code in that range. `E3100`,
/// `[UNS-1]`'s "requires an `unsafe` block", is in the range and is neither, so
/// no shape describes it and none should. The exception is named rather than
/// omitted so that a genuinely unclassified borrow error still fails the build.
/// See errata ERR-022; the specification needed no change.
/// `E3105`, `[UNS-10b]`'s "`UnsafeCell` is not permitted in `@static_safe`
/// code", is the same case as `E3100` and exempt for the same reason: it sits
/// in the range and is neither an ownership nor a borrow error, so the
/// classifier has nothing to say about it. `[UNS-10b]` also forbids any
/// `[DIA-*]` shape naming `UnsafeCell` as a fix, which is a second reason no
/// shape may describe it.
const NOT_OWNERSHIP_ERRORS: &[u16] = &[3100, 3105];

/// Every code in the ownership range that the compiler may emit and the
/// classifier cannot place. `[DIA-7a]` forbids emitting one, so this is worth
/// a test rather than a comment.
pub fn ownership_codes_are_all_classified() -> Vec<codes::Code> {
    codes::REGISTRY
        .iter()
        .copied()
        .filter(|c| {
            c.kind == codes::CodeKind::Error
                && c.subsystem == codes::Subsystem::Ownership
                && !NOT_OWNERSHIP_ERRORS.contains(&c.number)
                && shape_for(*c).is_none()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `[DIA-7a]` — "a code absent from this table MUST NOT be emitted", so
    /// every ownership code the registry declares must have a shape. This is
    /// the check `tools/rule_index.py` cannot make, because it reads the
    /// document rather than the compiler.
    #[test]
    fn every_ownership_code_has_a_shape() {
        let unclassified = ownership_codes_are_all_classified();
        assert!(
            unclassified.is_empty(),
            "these ownership codes have no diagnostic shape, which DIA-7a forbids: {}",
            unclassified
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    #[test]
    fn the_aliasing_rule_itself_is_reportable() {
        // The gap ERR-021 closed: `[BRW-1]` had no code, so a compiler that
        // rejected two overlapping mutable borrows could not say so.
        assert_eq!(shape_for(codes::E3021), Some(Shape::B3));
        assert_eq!(shape_for(codes::E3022), Some(Shape::B1));
    }
}
