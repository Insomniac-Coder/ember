#$ test: run-pass
#$ rules: CLO-3, CLO-1, TYP-17
#$ stdout: 3 3 5 12
# `[CLO-3]` — a lambda in a generic body that captures only concrete locals is
# a real function: an instance made while the body is checked calls it
# (D-257's review). Each closure has a name of its own in the C, however many
# generic bodies are checked with their parameters opaque.

fn apply(x: int, f: fn(int) -> int) -> int:
    return f(x)

fn g[T](t: T) -> int:
    k = 1
    return apply(2, fn(v) => v + k)

fn shift[T](x: T, by: int) -> int:
    return apply(1, fn(v) => v + by)

fn keep[T](x: T) -> int:
    k = 1
    h = fn(v: int) -> int => v + k
    return h(1)

fn twice[T](x: T) -> int:
    return apply(3, fn(v) => v * 2) + apply(3, fn(v) => v * 2)

fn main():
    m = 2
    h = fn(v: int) -> int => v + m
    print(g(5), shift("a", 2), h(1) + keep(0), "")
    println(twice(1.5))
