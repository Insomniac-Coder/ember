#$ test: compile-fail
#$ rules: FN-6, FN-6a
#$ error[E2228]: callable parameter mode mismatch at parameter 0: expected `mut`, found `borrowed`

fn apply(f: fn(mut i32) -> i32, mut value: i32) -> i32:
    return f(value)

fn main():
    value = 4
    println(apply(fn(x) => x, value))
