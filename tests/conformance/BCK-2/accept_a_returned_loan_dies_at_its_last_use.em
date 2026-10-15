#$ test: run-pass
#$ rules: BCK-2
#$ stdout: [5, 2, 3]
# `[BCK-2]` — the loan `r` carries is live only while `r` is used later: its last use comes before
# the push, so the push conflicts with nothing.

fn pick(mut xs: MutSpan[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    values = [1, 2]
    r = pick(values.as_mut_span())
    r = 5
    values.push(3)
    println(values)
