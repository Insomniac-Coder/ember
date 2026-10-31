#$ test: compile-fail
#$ rules: FN-6, FN-6a
#$ error[E2228]: callable parameter mode mismatch at parameter 0: expected `ref mut`, found `ref`

fn apply(f: fn(ref mut i32) -> i32, ref mut value: i32) -> i32:
    return f(value)

fn main():
    value: i32 = 4
    println(apply(fn(x) => x, value))
