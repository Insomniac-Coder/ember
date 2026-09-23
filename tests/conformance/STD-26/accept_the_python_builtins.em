#$ test: run-pass
#$ rules: STD-26, MOD-5
#$ profiles: debug, release, shipping
#$ stdout: 5 14 24
#$ true false false true
#$ 3 7 -2 9
#$ 4 4 1 10 0 5
# `len`, `sum` (with a start), `any`, `all`, and the two-argument `min`, `max`,
# with `abs` and `clamp`, as Python has them; each borrows its argument.

fn main():
    xs = [3, 1, 4, 1, 5]
    flags = [false, true, false]
    println(len(xs), sum(xs), sum(xs, start=10))
    println(any(flags), all(flags), any([false]), all([true, true]))
    println(min(3, 7), max(3, 7), min(-2, 5), max(9, -9))
    println(abs(-4), abs(4), abs(-1), clamp(15, 0, 10), clamp(-3, 0, 10), clamp(5, 0, 10))
