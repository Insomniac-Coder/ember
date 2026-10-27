#$ test: run-pass
#$ rules: R13, R3
#$ stdout: 3 2
## R13, R3: `if pair exists as (ref mut a, ref b):` opens an `Option` of a pair, each name with its
## own word.

fn main():
    pair: Option[(int, int)] = Some((1, 2))
    if pair exists as (ref mut a, ref b):
        a += b
    if pair exists as (x, y):
        println(x, y)
