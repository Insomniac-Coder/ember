#$ test: run-pass
#$ rules: BRW-21
#$ stdout: 5
## design rules: R8, R4
## R8 (g), verdict 53: the way out E3031's message gives: `record` reads the number its `Span` shows
## before it pushes to `SCORES`, so a span of `SCORES` may be handed to it. The nearest program to
## `reject_a_span_given_to_a_function_that_grows_what_it_shows.em` that is allowed.

static mut SCORES: Array[int] = []

fn record(xs: Span[int]):
    first = xs[0]
    SCORES.push(0)
    println(first)

fn main():
    SCORES.push(5)
    record(SCORES.as_span())
