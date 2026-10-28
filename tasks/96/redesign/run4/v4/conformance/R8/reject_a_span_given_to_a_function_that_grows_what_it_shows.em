#$ test: compile-fail
#$ rules: R8, R4
#$ help: write `first = xs[0]` above line 11 and `println(first)` in place of line 12
## R8 (b), (g), verdict 53: a view whose last use is an argument does not live across the call, and the
## callee is held to its promise: `record` uses its `Span` parameter after a push to `SCORES`, which may
## move the numbers the span shows, so a call that hands it a span of `SCORES` is refused (E3031).

static mut SCORES: Array[int] = []

fn record(xs: Span[int]) changes SCORES:
    SCORES.push(0)
    println(xs[0])

fn main() changes SCORES:
    SCORES.push(5)
    record(SCORES.as_span())    #$ error[E3031]: `record` changes `SCORES` and uses `xs` after `SCORES.push(0)` (line 11), which may move the numbers `xs` shows, and `SCORES.as_span()` points into `SCORES`; in `record`, read the number before the push: write `first = xs[0]` above line 11 and `println(first)` in place of line 12
