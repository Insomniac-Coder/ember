#$ test: run-pass
#$ rules: LT-7, LT-42, LT-1
#$ stdout: [2, 3]
# `[LT-7]` (D-220) — a capturing closure passed to a callable parameter: the
# call's result borrows what the callable type's source parameters lend it,
# here `s`, and the closure's captures only as far as the instance says
# (ODR-048). `return f(s)` was `E3062`, "points into `f`".

fn call(f: fn(Span[int]) -> Span[int], s: Span[int]) -> Span[int]:
    return f(s)

fn main():
    xs = [1, 2, 3]
    k = 1
    view = call(fn(v) => v[k..], xs)
    println(view)
