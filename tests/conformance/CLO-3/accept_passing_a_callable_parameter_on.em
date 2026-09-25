#$ test: run-pass
#$ rules: CLO-3, TYP-17
#$ stdout: 12 5 30 [2, 4] 7
# `[CLO-3]` — a callable parameter is a value like any other: a function may
# pass it on to another that takes a callable, from a generic body or a plain
# one, and a capturing closure survives being passed through two levels
# (D-257). A lambda written inside a generic body is made per instance.

fn apply[T](x: T, f: fn(T) -> int) -> int:
    return f(x)

fn twice[T](x: T, f: fn(T) -> int) -> int:
    return apply(x, f) + apply(x, f)

fn plain(x: int, f: fn(int) -> int) -> int:
    return apply(x, f)

fn outer(x: int, f: fn(int) -> int) -> int:
    return plain(x, f)

fn best[T](xs: Array[T], score: fn(T) -> int) -> int:
    top = 0
    for i in range(xs.len()):
        top = max(top, apply(xs[i], fn(v) => score(v)))
    return top

fn main():
    print(twice(3, fn(v) => v * 2), "")
    print(plain(4, fn(v) => v + 1), "")
    scale = 10
    print(outer(3, fn(v) => v * scale), "")
    xs = [1, 2, 3, 4]
    xs.retain(fn(v) => v % 2 == 0)
    print(xs, "")
    println(best([3, 7, 5], fn(v) => v))
