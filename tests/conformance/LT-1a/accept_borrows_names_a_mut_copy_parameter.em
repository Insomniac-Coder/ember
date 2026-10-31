#$ test: run-pass
#$ rules: LT-1a
#$ stdout: 5
# `[LT-1a]` (ODR-024) — `@borrows` may name a `mut` parameter, which is the
# caller's place, even of a `Copy` type.

fn slot(ref mut n: int) -> ref int from n:
    return ref n

fn main():
    x = 5
    r = slot(x)
    println(r)
