#$ test: run-pass
#$ rules: BRW-12, BRW-16
#$ stdout: 1
#$ 9
## design rules: R2, R8
## R2, R8 (g): the nearest program that is allowed: the number is changed after `r`'s last use.

fn main():
    counts = [1, 2]
    r = ref counts[0]
    println(r)
    counts[0] = 9
    println(counts[0])
