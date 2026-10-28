#$ test: run-pass
#$ rules: OWN-1
#$ stdout: 1 2
## design rules: R1
## R1: for a `Copy` type `=` copies: `n2` is a number of its own, and changing it leaves `n` as it was
## (AN-44's `n2 = n`).

fn main():
    n = 1
    n2 = n
    n2 += 1
    println(n, n2)
