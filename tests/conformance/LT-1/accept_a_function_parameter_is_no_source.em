#$ test: run-pass
#$ rules: LT-1, LT-7, CLO-3
#$ stdout: [1, 2] 2
# `[LT-1]` — a function-type parameter is not a source parameter: a call through
# `fn(fn() -> int, Span[int]) -> Span[int]` has one source, the span, so its result points into
# `xs` alone (rule 2) and `g` may be given another function while `r` is used.

fn main():
    xs = [1, 2]
    g = fn() => 1
    pick: fn(fn() -> int, Span[int]) -> Span[int] = fn(h, s) => s
    r = pick(g, xs)
    g = fn() => 2
    println(r, g())
