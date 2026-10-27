#$ test: run-pass
#$ rules: R2, R4
#$ stdout: 0
## R2, R4: a `ref mut` parameter is the exception: `=` on it writes the caller's place.

fn reset(ref mut n: int):
    n = 0

fn main():
    a = 5
    reset(ref mut a)
    println(a)
