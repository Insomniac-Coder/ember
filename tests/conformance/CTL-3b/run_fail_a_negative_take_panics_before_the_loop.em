#$ test: run-fail
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release, shipping
#$ stdout: before
#$ panics: take(-1): a count cannot be negative
# `[CTL-3b]` (ODR-089) — each count in a chain is checked once, before the
# loop, in the chain's order, and panics as its adapter does.

fn main():
    xs: Array[int] = [1, 2, 3]
    n = -1
    println("before")
    for x in xs.iter().skip(1).take(n):
        println(x)
