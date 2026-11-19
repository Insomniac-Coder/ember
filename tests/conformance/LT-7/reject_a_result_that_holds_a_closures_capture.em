#$ test: compile-fail
#$ rules: LT-7, LT-42
# `[LT-7]`, `[LT-42]` (D-220) — a closure that hands back a view it captured
# keeps what it captured borrowed through the call's result: `other` cannot
# be dropped while `r` is used.

fn call(f: fn(Span[int]) -> Span[int], s: Span[int]) -> Span[int]:
    return f(s)

fn main():
    xs = [1, 2, 3]
    other = [9, 9]
    r = call(fn(v) => other.as_span(), xs)
    mem.drop(other)
    println(r)  #$ error[E3060]: `other` was handed over, and a view of it is still used here
