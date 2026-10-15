#$ test: compile-fail
#$ rules: BCK-2, BCK-3
#$ error[E3022]: `values` is already mutably borrowed
# `[BCK-2]` — a result carries the loans of the arguments it borrows from, and a loan is live
# while a value carrying it is used later: `r` is used after the push, so the push conflicts.

fn pick(mut xs: MutSpan[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    values = [1, 2]
    r = pick(values.as_mut_span())
    values.push(3)
    r = 5
    println(values)
