#$ test: compile-fail
#$ rules: TYP-4
#$ error[E2020]: `+` cannot be applied to `i32` and `i64`
#$ note: `a as i64` makes them agree
#$ error[E2020]: `+` cannot be applied to `i64` and `f64`
#$ note: `b as f64` makes them agree
# `[TYP-4]` — no implicit conversion between numbers in an operator; the note
# names the exact cast on the narrower operand (an integer becomes a float).

fn main():
    a: i32 = 1
    b: i64 = 2
    f: f64 = 1.5
    c = a + b
    g = b + f
