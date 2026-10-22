#$ test: compile-fail
#$ rules: CLO-6a, CLO-6
## `[CLO-6]`, `[CLO-6a]` — a `once fn` value may be called once, so it is not a `fn(A) -> R`, which
## may be called many times: stored as one it is `E2020`, and given to a parameter `f: fn(A) -> R`
## it is `E3030` at the argument, whose help names `once fn`. The other way round is accepted.

fn many(f: fn() -> int) -> int:
    return f() + f()

fn main():
    h: once fn() -> int = fn() => 4
    k: fn() -> int = h    #$ error[E2020]: expected `fn() -> i64`, found `once fn() -> i64`
    m: once fn() -> int = fn() => 5
    println(many(m))    #$ error[E3030]: a `once fn() -> i64` value may be called only once
