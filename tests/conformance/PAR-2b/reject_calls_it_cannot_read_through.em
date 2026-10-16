#$ test: compile-fail
#$ rules: PAR-2b, PAR-2a
#$ error[E7011]: parallel loop has a loop-carried dependency on `xs`
#$ error[E7010]: the parallel loop gives a place it writes to `zero_from`, which comes back to itself, so the call cannot be read through
#$ error[E7010]: the parallel loop gives `xs` to a call, which may write any element of it
# `[PAR-2b]` — read through, `put_next` writes `xs` at `i + 1` while the loop writes it at `i`; a
# call that comes back to a function being read through is not read through, nor is a function
# the inline header does not hold (`@noinline`), and either, given the place, defeats the proof.

fn put_next(mut xs: MutSpan[int], at: int):
    xs[at + 1] = 1

fn shifts(mut xs: MutSpan[int]):
    @parallel
    for i in 0..xs.len() - 1:
        xs[i] = 0
        put_next(xs, i)

fn zero_from(mut xs: MutSpan[int], at: int):
    if at < xs.len():
        xs[at] = 0
        zero_from(xs, at + 1)

fn recurses(mut xs: MutSpan[int]):
    @parallel
    for i in 0..xs.len():
        zero_from(xs, i)

@noinline
fn opaque(mut xs: MutSpan[int], at: int):
    xs[at] = 1

fn hides(mut xs: MutSpan[int]):
    @parallel
    for i in 0..xs.len():
        opaque(xs, i)

fn main():
    pass
