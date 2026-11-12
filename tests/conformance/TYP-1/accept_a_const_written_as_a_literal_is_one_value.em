#$ test: run-pass
#$ rules: TYP-1, CT-1, LEX-24
#$ stdout: a b 1.0 2.5 5 -5 -1.5 7 5 true s -5
# `[TYP-1]` (D-721) — a `const` written as a literal is that value at each use, whatever type it is
# given (`const F: f64 = 1`, `-5` as an `i8`, a suffixed literal, a `char`); one that is not a literal as
# written is evaluated while compiling (`[CT-1]`, `accept_constant_expressions.em` of CT-1). Nothing
# reaches `E2130` but a `const` of a type that owns heap memory (`reject_what_a_constant_cannot_hold.em`),
# so the "a `const` must be a literal in this phase" path is gone.

const C = 'a'
const C2: char = 'b'
const F: f64 = 1
const F32: f32 = 2.5
const U: u8 = 5
const NEG: i8 = -5
const NEGF: f32 = -1.5
const U2 = 7_u16
const I = 5_i64
const B = true
const S = "s"
const I128: i128 = -5

fn main():
    println(C, C2, F, F32, U, NEG, NEGF, U2, I, B, S, I128)
