#$ test: run-pass
#$ rules: TYP-4, STD-19
#$ stdout: 3000000005
# `[TYP-4]` — of two whole numbers of different types, the one whose every value the other's type
# holds converts to it: an `i32` always fits an `i64`, so `i32 + i64` is an `i64` addition.

fn main():
    a: i32 = 5
    b: i64 = 3000000000
    sum = a + b
    c: i64 = sum
    println(c)
