#$ test: run-pass
#$ rules: R2
#$ stdout: 2
## R2: the nearest program that is allowed: a local of your own, made from the parameter.

fn bump(n: int) -> int:
    count = n
    count += 1
    return count

fn main():
    println(bump(1))
