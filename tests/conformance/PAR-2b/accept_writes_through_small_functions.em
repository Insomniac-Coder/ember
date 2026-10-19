#$ test: run-pass
#$ rules: PAR-2b, PAR-2
#$ profiles: debug, development
#$ stdout: [0, 3, 6, 9, 12, 15, 18, 21]
#$ stdout: [0, 4, 7, 10, 13, 16, 19, 22]
# `[PAR-2b]` — the analysis reads through a call to a function the inline header holds as if it
# were written in the loop: `triple` and `put` write `xs` at `i`, and `bump_next` reads and writes
# it at `i + 1` (its `at + 1`, given the caller's `i`), one offset, so each loop is independent.

fn put(mut xs: MutSpan[int], at: int, value: int):
    xs[at] = value

fn triple(mut xs: MutSpan[int], at: int):
    put(xs, at, at * 3)

fn bump_next(mut xs: MutSpan[int], at: int):
    if xs.len() > at + 1:
        xs[at + 1] = xs[at + 1] + 1

fn main():
    values = [0, 0, 0, 0, 0, 0, 0, 0]
    xs = values.as_mut_span()
    @parallel
    for i in 0..xs.len():
        triple(xs, i)
    println(values)
    ys = values.as_mut_span()
    @parallel
    for i in 0..ys.len() - 1:
        bump_next(ys, i)
    println(values)
