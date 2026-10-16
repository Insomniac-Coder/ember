#$ test: run-fail
#$ rules: RT-4, DIA-23
#$ profiles: debug
#$ stdout: 3
#$ panics: index 5 is out of bounds for a length of 2
#$ stderr: index 5 is out of bounds for a length of 2
#$ stderr: backtrace:
#$ stderr: 0: inner at
#$ stderr: 1: outer at
#$ stderr: 2: main at
# `[RT-4]`, `[DIA-23]` — a panic prints where it happened and, outside `shipping`, the program's
# functions from there back to `main`, each by its Ember name and where it is declared, never by a
# C symbol.

# Each is called from two places, so neither is inlined into its caller (a function called from
# one place is, and is then its caller's frame).
fn inner(xs: Array[int], i: int) -> int:
    return xs[i]

fn outer(xs: Array[int], i: int) -> int:
    return inner(xs, i) + inner(xs, 0)

fn main():
    xs = [1, 2]
    println(outer(xs, 1))
    println(outer(xs, 5))
