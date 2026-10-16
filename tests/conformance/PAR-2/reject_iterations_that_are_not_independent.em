#$ test: compile-fail
#$ rules: PAR-2
#$ error[E7010]: a `@parallel` loop's body may not `break` out of it
#$ error[E7010]: a `@parallel` loop's body may not `return`
#$ error[E7010]: the parallel loop writes `seen`, which its chunks share
#$ error[E7010]: the parallel loop writes `xs`, a `Array[i64]`; it may write a `MutSpan` or an `SoA` column
#$ error[E7010]: the parallel loop writes `xs`, and reaches it here at an index that is not `i` plus a constant
#$ error[E7010]: the parallel loop gives `xs` to a call, which may write any element of it
# `[PAR-2]` — a `@parallel` loop's chunks run at once, so its iterations must be independent: no
# `break` or `return`; nothing of the enclosing code written but a `MutSpan` or an `SoA` column,
# and each of those only at `i` plus a constant; and no such place handed to a call, which might
# write any element of it.

fn leaves(mut xs: MutSpan[int]):
    @parallel
    for i in 0..xs.len():
        if xs[i] < 0:
            break
        xs[i] = 1

fn gives_up(mut xs: MutSpan[int]) -> int:
    @parallel
    for i in 0..xs.len():
        if xs[i] < 0:
            return i
        xs[i] = 1
    return 0

fn counts(xs: Span[int]) -> int:
    seen = 0
    @parallel
    for i in 0..xs.len():
        seen += xs[i]
    return seen

fn grows(mut xs: Array[int]):
    @parallel
    for i in 0..xs.len():
        xs[i] = 2

fn scatters(mut xs: MutSpan[int], at: Span[int]):
    @parallel
    for i in 0..xs.len():
        xs[at[i]] = 2

@noinline
fn fill(mut xs: MutSpan[int]):
    xs[0] = 1

fn hands_off(mut xs: MutSpan[int]):
    @parallel
    for i in 0..xs.len():
        fill(xs)

fn main():
    pass
