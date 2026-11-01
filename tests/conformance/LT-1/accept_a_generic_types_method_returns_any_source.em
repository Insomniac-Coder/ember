#$ test: run-pass
#$ rules: LT-1, LT-22
#$ stdout: k [1, 2]
#$ 2 [1, 2]
# `[LT-1]` (verdict 138, ODR-069) — a method may return a view of `this` and of any source parameter
# with nothing written, a generic type's method too: ODR-069 makes a parameter whose type names the
# result's type parameter a source beside `this`, and takes no other source away.

[T]
struct Holder:
    item: T

    fn pick(k: T, other: Span[int]) -> (T, Span[int]):
        return (k, other)

fn main():
    xs = [1, 2]
    h = Holder(item="a")
    (a, b) = h.pick("k", xs)
    println(a, b)
    g = Holder(item=1)
    (c, d) = g.pick(2, xs)
    println(c, d)
