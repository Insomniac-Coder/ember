#$ test: run-pass
#$ rules: CLO-3
## "Calling: `f(args)`. A parameter declared `f: fn(A) -> R` is a generic over
## `Callable` (static dispatch, monomorphised)."

fn compose(f: fn(i32) -> i32, g: fn(i32) -> i32, v: i32) -> i32:
    return g(f(v))

fn main():
    println(compose(fn(x) => x + 1, fn(x) => x * 10, 4))
#$ stdout: 50
