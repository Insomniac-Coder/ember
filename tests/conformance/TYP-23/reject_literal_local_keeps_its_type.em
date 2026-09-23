#$ test: compile-fail
#$ rules: TYP-23, LEX-16
#$ error[E2020]: expected `i32`, found `i64`
#$ help: declare it as `total: i32 = ...` to make it `i32`
# ODR-022: `total = 0` declares an `int` at the declaration; a later use that
# needs another type does not change it, and the help names the annotation.

fn sum_to(n: i32) -> i32:
    total = 0
    for i in 0..n:
        total += i
    return total

fn main():
    println(sum_to(4))
