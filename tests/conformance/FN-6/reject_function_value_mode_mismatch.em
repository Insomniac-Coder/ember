#$ test: compile-fail
#$ rules: FN-6, FN-6a, TYP-18
#$ error[E2228]: callable parameter mode mismatch at parameter 0: expected `mut`, found `borrowed`

fn identity(value: i32) -> i32:
    return value

fn main():
    operation: fn(mut i32) -> i32 = identity
    value = 1
    println(operation(value))
