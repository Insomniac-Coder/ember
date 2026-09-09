#$ test: run-pass
#$ rules: FN-6, CLO-3
## "Functions are values of a unique zero-sized function type; they coerce to
## `fn(A) -> R` (the generic callable bound)."

fn double(x: i32) -> i32:
    return x * 2

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn twice(f: fn(i32) -> i32, v: i32) -> i32:
    return f(f(v))

fn main():
    println(apply(double, 21))
    println(twice(double, 3))
#$ stdout: 42
#$ 12
