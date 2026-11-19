#$ test: compile-fail
#$ rules: TYP-4, STD-19
# `[TYP-4]` — `a + b` for an `int` and a `u64` that may be any of their values is `E2020`: the `int`
# may be negative and the `u64` may pass `int`'s top. No value converts implicitly between scalar
# types in an operator but the one whose every value the other holds.

fn add(a: int, b: u64) -> u64:
    return a + b                    #$ error[E2020]: `+` cannot be applied to `i64` and `u64`

fn main():
    println(add(1, 2))
