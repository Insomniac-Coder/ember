# Clause PAR-3.1: "`@parallel(reduce=[total: +, best: max])` declares the variables the loop's
# iterations combine, whole numbers included (...). Inside the body each chunk has its own
# accumulator, which starts and is combined as a task's does (`[THR-21]`)"
#$ test: run-pass
#$ rules: PAR-3, PAR-5, THR-21
#$ stdout: 10 4
# The spec's declaration.

fn stats(xs: Span[int]) -> (int, int):
    total = 0
    best = 0
    @parallel(chunk=2, reduce=[total: +, best: max])
    for i in 0..xs.len():
        total += xs[i]
        best = max(best, xs[i])
    return (total, best)

fn main():
    r = stats([1, 4, 2, 3])
    println(r.0, r.1)
