#$ test: run-pass
#$ rules: CTL-3b, CTL-3
#$ profiles: debug, release, shipping
#$ stdout: 6.0 12 3.0 21
# `[CTL-3b]` — a counted loop is an induction-variable loop whatever the host
# C compiler's optimiser. For MSVC, which does not unroll a loop setting a
# variable of the whole function, the loop runs on a block-local copy of each
# number it sets and copies it back (ADR-086); the results are the loop's,
# on every compiler: a float total, an integer one, one read after a loop
# that also reads it inside, and two totals in one loop.

fn float_total(xs: Array[f64]) -> f64:
    s = 0.0
    for i in 0..len(xs):
        s += xs[i]
    return s

fn int_total(xs: Array[int]) -> int:
    s = 0
    for i in 0..len(xs):
        s = s + xs[i]
    return s

fn two_totals(xs: Array[f64]) -> (f64, int):
    s = 0.0
    n = 0
    for i in 0..len(xs):
        x = xs[i]
        s += x * 0.5
        n += i
    return (s, n)

fn main():
    fs: Array[f64] = [1.0, 2.0, 3.0]
    ns: Array[int] = [3, 4, 5]
    halves, count = two_totals(fs)
    grow = 0
    for i in 0..7:
        grow = grow + i
    println(float_total(fs), int_total(ns), halves, grow + count - 3)
