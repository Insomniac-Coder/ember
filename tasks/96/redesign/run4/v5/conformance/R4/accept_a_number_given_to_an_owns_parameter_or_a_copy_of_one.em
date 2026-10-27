#$ test: run-pass
#$ rules: R4, R1
#$ stdout: 1 2
#$ stdout: 6
## R4, R1: `take(copy of n)` gives `take` a copy and the caller keeps its own `n`; `take(k)` hands `k`
## over with no word, and `k` is not used again (verdict 84: `owns` hands over, and `copy of` copies,
## for every type).

fn take(owns x: int) -> int:
    return x + 1

fn main():
    n = 1
    m = take(copy of n)
    println(n, m)
    k = 5
    println(take(k))
