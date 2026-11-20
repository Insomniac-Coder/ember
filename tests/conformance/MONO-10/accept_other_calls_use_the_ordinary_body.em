# Clause MONO-10.2: "calls that do not name one variable twice use the ordinary body, where a `ref
# mut` parameter is the only way to its place (`[CG-C-4]`)"
#$ test: run-pass
#$ rules: MONO-10, BRW-22
#$ stdout: 1
#$ stdout: 1 10
#$ stdout: 1 1
# `tally(ref mut x)` and `add_both(ref mut p, ref mut q)` name two places: the ordinary bodies.

static mut TOTAL: int = 0

fn add_both(ref mut a: int, ref mut b: int):
    a += 1
    b += 1

fn tally(ref mut n: int):
    n += 1
    TOTAL += 10
    println(n)

fn main():
    x = 0
    tally(ref mut x)
    println(x, TOTAL)
    p = 0
    q = 0
    add_both(ref mut p, ref mut q)
    println(p, q)
