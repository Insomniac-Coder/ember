#$ test: run-pass
#$ rules: BRW-12, FN-1
#$ stdout: 0
## design rules: R2, R4
## R2, R4: a `ref mut` parameter is the exception: `=` on it writes the caller's place.

fn reset(ref mut n: int):
    n = 0

fn main():
    a = 5
    reset(ref mut a)
    println(a)
