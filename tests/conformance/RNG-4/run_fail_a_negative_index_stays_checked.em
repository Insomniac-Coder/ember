#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 6
#$ panics: index -1 is out of bounds for a length of 4
# `[RNG-4]` — `k < xs.len()` bounds `k` above but not below: a negative index
# keeps its check.

fn at(xs: Array[int], k: int) -> int:
    if k < xs.len():
        return xs[k]
    return -1

fn main():
    xs: Array[int] = [5, 6, 7, 8]
    println(at(xs, 1))
    println(at(xs, -1))
