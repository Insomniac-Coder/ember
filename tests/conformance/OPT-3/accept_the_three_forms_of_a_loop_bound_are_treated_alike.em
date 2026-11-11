#$ test: run-pass
#$ rules: OPT-3, OPT-2
#$ profiles: release
#$ stdout: 15 15 15
#$ assert-c-count: contains("ember_panic_bounds(") == 0
# `[OPT-3]` — the loop bound may be written `s.len()`, a separate local the loop does not assign, or
# a constant; the three are treated alike (no bounds check is left in any of the three loops).

const N: int = 5

fn by_len(xs: Span[int]) -> int:
    total = 0
    for i in 0..xs.len():
        total += xs[i]
    return total

fn by_local(xs: Span[int]) -> int:
    n = xs.len()
    total = 0
    for i in 0..n:
        total += xs[i]
    return total

fn by_const(xs: [int; 5]) -> int:
    total = 0
    for i in 0..N:
        total += xs[i]
    return total

fn main():
    xs = [1, 2, 3, 4, 5]
    fixed: [int; 5] = [1, 2, 3, 4, 5]
    println(by_len(xs), by_local(xs), by_const(fixed))
