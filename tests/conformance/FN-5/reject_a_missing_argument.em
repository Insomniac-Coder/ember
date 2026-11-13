#$ test: compile-fail
#$ rules: FN-5
#$ error[E2021]: `three(a=1, b=2)` gives nothing for `c`, which has no default: `fn three(a: int, b: int = 1, c: int) -> int`
# `[FN-5]` — a named argument may skip a defaulted parameter, but not one
# without a default.

fn two(a: int, b: int = 1, c: int = 2) -> int:
    return a + b + c

fn three(a: int, b: int = 1, c: int) -> int:
    return a + b + c

fn main():
    println(two(1, c=5))
    println(three(1, c=2), three(a=1, b=2))
