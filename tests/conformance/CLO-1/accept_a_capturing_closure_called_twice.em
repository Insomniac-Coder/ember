#$ test: run-pass
#$ rules: CLO-1, CLO-3
# `[CLO-3]`'s bound is `Callable`, not `CallableOnce`, so a closure reached
# through a plain `f: fn(A) -> R` parameter may be called as often as the body
# likes — the environment is read, never consumed.

fn twice(f: fn(i32) -> i32, v: i32) -> i32:
    return f(f(v))

fn main():
    k = 5
    println(twice(fn(x) => x + k, 1))
#$ stdout: 11
