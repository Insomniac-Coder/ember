//! `[TYP-41]` (ODR-066) — the normal form of a const generic argument.
//!
//! An integer argument that names the declaration's `const` parameters is
//! compared by its normal form: a sum of terms, each an integer coefficient
//! times a product of parameters and opaque terms, with like terms collected
//! and zero terms dropped, computed in mathematical integers. `+`, `-`,
//! multiplication and parentheses are what the form sees through; any other
//! operation is an opaque term, equal to another only when it is the same
//! operation on operands whose normal forms are equal. So `N + 1` and `1 + N`
//! are one form, and `2 * N` and `N + N`; `N / 2 * 2` and `N` are not. A form
//! with no parameter is a number, and every operation on numbers is computed.

use std::fmt;

use ember_span::Symbol;

/// A sum of terms, sorted by their atoms, with no zero coefficient; the
/// constant term is the one with no atoms.
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Default)]
pub struct ConstNorm {
    pub terms: Vec<(Vec<ConstAtom>, i128)>,
}

/// A factor of a term.
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ConstAtom {
    /// The declaration's `const` parameter at this index, by its name.
    Param(u32, Symbol),
    /// An operation the form does not see through, on its operands' forms.
    Opaque(ConstOp, Vec<ConstNorm>),
}

/// The operations a normal form keeps whole (`[TYP-41]` rule 2).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ConstOp {
    FloorDiv,
    Rem,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
}

/// Why a form could not be computed: `[TYP-41]` rule 1 makes it `E6004`
/// where the argument is written.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ConstError {
    Overflow,
    DivisionByZero,
    NegativeShift,
}

impl ConstNorm {
    pub fn constant(value: i128) -> ConstNorm {
        let mut out = ConstNorm::default();
        if value != 0 {
            out.terms.push((Vec::new(), value));
        }
        out
    }

    pub fn param(index: u32, name: Symbol) -> ConstNorm {
        ConstNorm { terms: vec![(vec![ConstAtom::Param(index, name)], 1)] }
    }

    /// The number this form is, when it names no parameter.
    pub fn as_constant(&self) -> Option<i128> {
        match self.terms.as_slice() {
            [] => Some(0),
            [(atoms, value)] if atoms.is_empty() => Some(*value),
            _ => None,
        }
    }

    /// The parameter this form is exactly (`N`, coefficient one, nothing else).
    pub fn as_param(&self) -> Option<u32> {
        match self.terms.as_slice() {
            [(atoms, 1)] => match atoms.as_slice() {
                [ConstAtom::Param(index, _)] => Some(*index),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn mentions_params(&self) -> bool {
        self.terms.iter().any(|(atoms, _)| {
            atoms.iter().any(|atom| match atom {
                ConstAtom::Param(..) => true,
                ConstAtom::Opaque(_, operands) => operands.iter().any(ConstNorm::mentions_params),
            })
        })
    }

    fn from_terms(mut terms: Vec<(Vec<ConstAtom>, i128)>) -> Result<ConstNorm, ConstError> {
        for (atoms, _) in &mut terms {
            atoms.sort();
        }
        terms.sort_by(|a, b| a.0.cmp(&b.0));
        let mut out: Vec<(Vec<ConstAtom>, i128)> = Vec::new();
        for (atoms, value) in terms {
            match out.last_mut() {
                Some((last, sum)) if *last == atoms => *sum = sum.checked_add(value).ok_or(ConstError::Overflow)?,
                _ => out.push((atoms, value)),
            }
        }
        out.retain(|(_, value)| *value != 0);
        Ok(ConstNorm { terms: out })
    }

    pub fn add(&self, other: &ConstNorm) -> Result<ConstNorm, ConstError> {
        ConstNorm::from_terms(self.terms.iter().chain(&other.terms).cloned().collect())
    }

    pub fn neg(&self) -> Result<ConstNorm, ConstError> {
        let terms = self
            .terms
            .iter()
            .map(|(atoms, value)| Ok((atoms.clone(), value.checked_neg().ok_or(ConstError::Overflow)?)))
            .collect::<Result<Vec<_>, ConstError>>()?;
        ConstNorm::from_terms(terms)
    }

    pub fn sub(&self, other: &ConstNorm) -> Result<ConstNorm, ConstError> {
        self.add(&other.neg()?)
    }

    pub fn mul(&self, other: &ConstNorm) -> Result<ConstNorm, ConstError> {
        let mut terms = Vec::new();
        for (a_atoms, a) in &self.terms {
            for (b_atoms, b) in &other.terms {
                let atoms = a_atoms.iter().chain(b_atoms).cloned().collect();
                terms.push((atoms, a.checked_mul(*b).ok_or(ConstError::Overflow)?));
            }
        }
        ConstNorm::from_terms(terms)
    }

    /// An operation the form keeps whole, computed when both operands are
    /// numbers. `//` and `%` floor, as Ember's integer operators do.
    pub fn opaque(op: ConstOp, left: &ConstNorm, right: &ConstNorm) -> Result<ConstNorm, ConstError> {
        let (Some(a), Some(b)) = (left.as_constant(), right.as_constant()) else {
            return Ok(ConstNorm { terms: vec![(vec![ConstAtom::Opaque(op, vec![left.clone(), right.clone()])], 1)] });
        };
        let value = match op {
            ConstOp::FloorDiv | ConstOp::Rem if b == 0 => return Err(ConstError::DivisionByZero),
            ConstOp::FloorDiv => {
                let q = a.checked_div(b).ok_or(ConstError::Overflow)?;
                if (a % b != 0) && ((a < 0) != (b < 0)) { q - 1 } else { q }
            }
            ConstOp::Rem => {
                let r = a.checked_rem(b).ok_or(ConstError::Overflow)?;
                if r != 0 && ((r < 0) != (b < 0)) { r + b } else { r }
            }
            ConstOp::Shl | ConstOp::Shr if !(0..128).contains(&b) => return Err(ConstError::NegativeShift),
            ConstOp::Shl => a.checked_shl(b as u32).ok_or(ConstError::Overflow)?,
            ConstOp::Shr => a >> b,
            ConstOp::BitAnd => a & b,
            ConstOp::BitOr => a | b,
            ConstOp::BitXor => a ^ b,
        };
        Ok(ConstNorm::constant(value))
    }

    /// This form with each parameter replaced by what `value` says it is
    /// (left as it is where `value` says nothing), computed again.
    pub fn substitute(&self, value: &impl Fn(u32) -> Option<ConstNorm>) -> Result<ConstNorm, ConstError> {
        let mut sum = ConstNorm::default();
        for (atoms, coefficient) in &self.terms {
            let mut product = ConstNorm::constant(*coefficient);
            for atom in atoms {
                let factor = match atom {
                    ConstAtom::Param(index, name) => value(*index).unwrap_or_else(|| ConstNorm::param(*index, *name)),
                    ConstAtom::Opaque(op, operands) => {
                        let left = operands[0].substitute(value)?;
                        let right = operands[1].substitute(value)?;
                        ConstNorm::opaque(*op, &left, &right)?
                    }
                };
                product = product.mul(&factor)?;
            }
            sum = sum.add(&product)?;
        }
        Ok(sum)
    }
}

impl fmt::Display for ConstNorm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.terms.is_empty() {
            return write!(f, "0");
        }
        // The constant term last, as it is usually written: `2 * N + 1`.
        let mut ordered: Vec<&(Vec<ConstAtom>, i128)> = self.terms.iter().filter(|(atoms, _)| !atoms.is_empty()).collect();
        ordered.extend(self.terms.iter().filter(|(atoms, _)| atoms.is_empty()));
        for (index, (atoms, coefficient)) in ordered.into_iter().enumerate() {
            let negative = *coefficient < 0;
            match (index, negative) {
                (0, true) => write!(f, "-")?,
                (0, false) => {}
                (_, true) => write!(f, " - ")?,
                (_, false) => write!(f, " + ")?,
            }
            let magnitude = coefficient.unsigned_abs();
            if atoms.is_empty() {
                write!(f, "{magnitude}")?;
                continue;
            }
            if magnitude != 1 {
                write!(f, "{magnitude} * ")?;
            }
            for (at, atom) in atoms.iter().enumerate() {
                if at > 0 {
                    write!(f, " * ")?;
                }
                match atom {
                    ConstAtom::Param(_, name) => write!(f, "{name}")?,
                    ConstAtom::Opaque(op, operands) => {
                        let symbol = match op {
                            ConstOp::FloorDiv => "//",
                            ConstOp::Rem => "%",
                            ConstOp::Shl => "<<",
                            ConstOp::Shr => ">>",
                            ConstOp::BitAnd => "&",
                            ConstOp::BitOr => "|",
                            ConstOp::BitXor => "^",
                        };
                        write!(f, "({} {symbol} {})", operands[0], operands[1])?
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n() -> ConstNorm {
        ConstNorm::param(0, Symbol::intern("N"))
    }

    #[test]
    fn the_rules_examples() {
        let one = ConstNorm::constant(1);
        let two = ConstNorm::constant(2);
        // `N + 1` and `1 + N` are one form, and `2 * N` and `N + N`.
        assert_eq!(n().add(&one).unwrap(), one.add(&n()).unwrap());
        assert_eq!(two.mul(&n()).unwrap(), n().add(&n()).unwrap());
        // `N // 2 * 2` and `N` are not.
        let halved = ConstNorm::opaque(ConstOp::FloorDiv, &n(), &two).unwrap();
        assert_ne!(halved.mul(&two).unwrap(), n());
        // Zero terms are dropped: `N - N` is the number 0.
        assert_eq!(n().sub(&n()).unwrap().as_constant(), Some(0));
        // Substitution computes: `N // 2 * 2` at N = 7 is 6.
        let at_seven = halved.mul(&two).unwrap().substitute(&|_| Some(ConstNorm::constant(7))).unwrap();
        assert_eq!(at_seven.as_constant(), Some(6));
        assert_eq!(format!("{}", two.mul(&n()).unwrap().add(&one).unwrap()), "2 * N + 1");
        assert_eq!(ConstNorm::opaque(ConstOp::FloorDiv, &one, &ConstNorm::constant(0)), Err(ConstError::DivisionByZero));
        assert_eq!(ConstNorm::opaque(ConstOp::FloorDiv, &ConstNorm::constant(-7), &two).unwrap().as_constant(), Some(-4));
    }
}
