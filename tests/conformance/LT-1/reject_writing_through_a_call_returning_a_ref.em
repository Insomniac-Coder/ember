#$ test: compile-fail
#$ rules: LT-1, BRW-1
# `[LT-1]` — a call whose result type is `ref T` is a place, one that only reads: writing through it
# is refused.

fn peek(xs: Array[int]) -> ref int:
    return ref xs[0]

fn main():
    xs = [1, 2]
    peek(xs) += 1    #$ error[E3021]: cannot write through a shared reference
    println(xs)
