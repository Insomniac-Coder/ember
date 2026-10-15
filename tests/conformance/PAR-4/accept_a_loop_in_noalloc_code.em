#$ test: run-pass
#$ rules: PAR-4, PAR-1
#$ profiles: debug, release
#$ stdout: 20000
# `[PAR-4]` — a `@parallel` loop allocates nothing: its record is in the caller's frame, its helper
# jobs fit their slots, and its reductions' window is the caller's too; so it is `@noalloc`-clean.

@noalloc
fn total(mut xs: MutSpan[int]) -> int:
    sum = 0
    @parallel(chunk=1000, reduce=[sum: +])
    for i in 0..xs.len():
        xs[i] = xs[i] + 1
        sum += xs[i]
    return sum

fn main():
    values: Array[int] = []
    for i in range(10000):
        values.push(1)
    println(total(values.as_mut_span()))
