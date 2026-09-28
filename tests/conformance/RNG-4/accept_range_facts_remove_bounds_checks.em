#$ test: run-pass
#$ rules: RNG-4, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 26 26 7 -1 -1
#$ stdout: 82
#$ assert-c-count: contains("ember_panic_bounds(") == 0
# `[RNG-4]` — each index is known to be below its length where it is used:
# a `for` over `0..xs.len()`, a `while i < xs.len()` whose body leaves `xs`
# alone, a guard `k >= 0 and k < xs.len()`, and `i % 8` into an array of 8.
# No bounds check is left, and no loop needs an unchecked copy.

fn sum_for(xs: Array[int]) -> int:
    total = 0
    for i in 0..xs.len():
        total += xs[i]
    return total

fn sum_while(xs: Array[int]) -> int:
    total = 0
    i = 0
    while i < xs.len():
        total += xs[i]
        i += 1
    return total

fn at_or(xs: Array[int], k: int) -> int:
    if k >= 0 and k < xs.len():
        return xs[k]
    return -1

fn ring(slots: [int; 8], n: int) -> int:
    total = 0
    for i in 0..n:
        total += slots[i % 8]
    return total

fn main():
    xs: Array[int] = [5, 6, 7, 8]
    println(sum_for(xs), sum_while(xs), at_or(xs, 2), at_or(xs, 9), at_or(xs, -1))
    println(ring([1, 2, 3, 4, 5, 6, 7, 8], 20))
