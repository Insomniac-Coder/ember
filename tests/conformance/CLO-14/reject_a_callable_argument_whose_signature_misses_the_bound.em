#$ test: compile-fail
#$ rules: CLO-14, TYP-17, FN-6a
# D-423 — a callable argument's signature is checked against the bound at
# the call, and the call is the error: `apply(fn(v: str) => 1, 2)` was
# reported in `apply`'s body, and `xs.iter().map(takes_str)` inside
# `Map.next` in `std/src/core.em`.

fn apply[F: fn(int) -> int](f: F, x: int) -> int:
    return f(x)

fn takes_str(s: str) -> int:
    return 1

fn yes(v: int) -> bool:
    return true

fn main():
    println(apply(fn(v: str) => 1, 2))    #$ error[E2020]: `apply` cannot take `fn(str) -> i64` where it expects `fn(i64) -> i64`
    println(apply(yes, 2))    #$ error[E2020]: `apply` cannot take `fn(i64) -> bool` where it expects `fn(i64) -> i64`
    xs = [1, 2, 3]
    for v in xs.iter().map(takes_str):    #$ error[E2020]: `map` cannot take `fn(str) -> i64` where it expects `fn(ref i64) -> i64`
        println(v)
