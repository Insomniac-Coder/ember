#$ test: compile-fail
#$ rules: FN-6, FN-6a, TYP-18
#$ error[E2020]: expected `fn(mut i32) -> i32`, found `fn(i32) -> i32`

fn identity(value: i32) -> i32:
    return value

fn main():
    operation: fn(mut i32) -> i32 = identity
    value = 1
    println(operation(value))
