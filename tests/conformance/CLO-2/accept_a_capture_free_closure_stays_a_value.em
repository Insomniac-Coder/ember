#$ test: run-pass
#$ rules: CLO-2, CLO-1, FN-6
# The other side of the inference: a closure that captures nothing is "a plain
# value type" (`[CLO-1]`), so it stays an ordinary `fn(A) -> R` — a function
# pointer, assignable to a `fn`-typed local, which a capturing one is not.
# `[FN-6]` is what keeps `fn(A) -> R` a real type outside parameter position.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    step: fn(i32) -> i32 = fn(x) => x * 3
    println(apply(step, 4))
    println(apply(fn(x) => x - 1, 10))
#$ stdout: 12
#$ 9
