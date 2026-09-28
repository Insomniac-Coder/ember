#$ test: run-fail
#$ rules: OPT-2, RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 10
#$ panics: index 4 is out of bounds for a length of 4
# `[OPT-2]` — `n - 1 - i` is at most `n - 1`; with `n` past the length the
# entry test fails and the checked loop panics at the first index.

fn rev_sum(xs: Array[int], n: int) -> int:
    total = 0
    for i in 0..n:
        total += xs[n - 1 - i]
    return total

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    println(rev_sum(xs, 4))
    println(rev_sum(xs, 5))
