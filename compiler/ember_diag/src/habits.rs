//! `[DIA-21]` — the Python habits, each a catalogued shape (ODR-179): `P1`, `P2`, … in the order of
//! `[DIA-21]`'s table, then the rows the owner's rulings of 2026-10-22 added (ODR-180..187) and the
//! habits the first-week corpus met in no catalogue (`[TST-9]`), each with the codes it is reported
//! under. A diagnostic answering one carries its id ([`crate::Diagnostic::habit`]); `--json` gives it
//! as `"shape"`, which `tests/ui/python/<id>/` and `tests/firstweek/marks.json` name.

use std::fmt;

use crate::codes::{self, Code};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Habit {
    P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18, P19, P20,
    P21, P22, P23, P24, P25, P26, P27, P28, P29, P30,
}

impl fmt::Display for Habit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Habit {
    pub const ALL: [Habit; 30] = [
        Habit::P1, Habit::P2, Habit::P3, Habit::P4, Habit::P5, Habit::P6, Habit::P7, Habit::P8, Habit::P9, Habit::P10,
        Habit::P11, Habit::P12, Habit::P13, Habit::P14, Habit::P15, Habit::P16, Habit::P17, Habit::P18, Habit::P19,
        Habit::P20, Habit::P21, Habit::P22, Habit::P23, Habit::P24, Habit::P25, Habit::P26, Habit::P27, Habit::P28,
        Habit::P29, Habit::P30,
    ];

    /// What the program wrote: the table's "Written" column.
    pub fn written(self) -> &'static str {
        match self {
            Habit::P1 => "`True`, `False`, `None` as a value where no `Option` is expected",
            Habit::P2 => "`def f():`",
            Habit::P3 => "`str(x)`, `int(x)`, `float(x)`",
            Habit::P4 => "`len(s)` for a string",
            Habit::P5 => "`min(xs)`, `max(xs)` of one collection",
            Habit::P6 => "a Python method name or keyword argument (`xs.append(x)`, `s.strip()`, `\", \".join(xs)`, `m.get(k, d)`, `c.isdigit()`, …)",
            Habit::P7 => "`xs[a:b]`",
            Habit::P8 => "`if xs:`",
            Habit::P9 => "`let x = 5`, `var x = 5`",
            Habit::P10 => "`x == None`",
            Habit::P11 => "`with open(p) as f:`",
            Habit::P12 => "`raise e`, `try:`, `except E:`, `finally:`",
            Habit::P13 => "`lambda x: e`",
            Habit::P14 => "`self` missing from a method's parameters",
            Habit::P15 => "`a / b` on integers",
            Habit::P16 => "`if x = 5:`",
            Habit::P17 => "`case p:` inside `match`",
            Habit::P18 => "`global x`, `nonlocal x`",
            Habit::P19 => "`*args`, `**kwargs` in a signature",
            Habit::P20 => "`\"%d\" % n`, `\"{}\".format(n)`",
            Habit::P21 => "`Err(\"message\")` in a function returning `Result[T]` (ODR-180)",
            Habit::P22 => "an interface name as a type: `s: Shape`, `Array[Shape]` (ODR-182)",
            Habit::P23 => "sequence repetition: `s * n`, `[x] * n` (ODR-185)",
            Habit::P24 => "a single-quoted string: `'abc'`, `''` (ODR-186)",
            Habit::P25 => "a list operation on an iterator: `s.split(\" \")[0]`, `.len()` (ODR-187)",
            Habit::P26 => "a `str` value where a `String` is kept (`[TXT-9]`)",
            Habit::P27 => "a `char` where a `str` or `String` is expected, or compared with one",
            Habit::P28 => "an assignment or `pass` as a `=>` arm",
            Habit::P29 => "indexing a string: `s[i]`",
            Habit::P30 => "indexing a tuple: `p[1]`",
        }
    }

    /// The codes a diagnostic answering this habit may carry.
    pub fn codes(self) -> &'static [Code] {
        match self {
            Habit::P1 => &[codes::E1010, codes::E2020, codes::E2060],
            Habit::P2 | Habit::P7 | Habit::P9 | Habit::P12 | Habit::P13 | Habit::P17 | Habit::P18 | Habit::P19 | Habit::P28 => {
                &[codes::E0100]
            }
            Habit::P3 | Habit::P11 => &[codes::E1010],
            Habit::P4 => &[codes::E2073],
            Habit::P5 => &[codes::E2020],
            Habit::P6 => &[codes::E2072, codes::E2020],
            Habit::P8 => &[codes::E2035],
            Habit::P10 => &[codes::E2040],
            Habit::P14 => &[codes::E1010],
            Habit::P15 => &[codes::E2240],
            Habit::P16 => &[codes::E2036],
            Habit::P20 => &[codes::E2020, codes::E2072],
            Habit::P21 | Habit::P23 | Habit::P26 | Habit::P27 | Habit::P29 | Habit::P30 => &[codes::E2020],
            Habit::P22 => &[codes::E2173, codes::E2020],
            Habit::P24 => &[codes::E0008],
            Habit::P25 => &[codes::E2072, codes::E2020],
        }
    }

    pub fn from_id(id: &str) -> Option<Habit> {
        Habit::ALL.into_iter().find(|habit| habit.to_string() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_in_the_table_order() {
        for (at, habit) in Habit::ALL.into_iter().enumerate() {
            assert_eq!(habit.to_string(), format!("P{}", at + 1));
            assert_eq!(Habit::from_id(&habit.to_string()), Some(habit));
            assert!(!habit.codes().is_empty(), "{habit} has a code (ODR-179)");
        }
    }
}
