#$ test: compile-fail
#$ rules: CT-1, CT-3
# `[CT-1]` — constants may name each other in any order, but a value that
# needs itself could never be worked out: `E6001`, reported where the cycle
# closes.

const A: i32 = B + 1
const B: i32 = A * 2    #$ error[E6001]: the value of `A` depends on itself
const ALONE: i64 = ALONE    #$ error[E6001]: the value of `ALONE` depends on itself

fn main():
    println(A, B, ALONE)
