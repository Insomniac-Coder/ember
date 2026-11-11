#$ test: run-fail
#$ rules: PRF-1, PRF-4
#$ profiles: debug, development, release
#$ panics: index 3 is out of bounds for a length of 3
# `[PRF-1]` — every profile performs the same safety checks while the program runs and stops in the
# same places with the same messages: the bounds check fails in `release` as in `debug`.

fn main():
    xs: Array[int] = [1, 2, 3]
    i = xs.len()
    println(xs[i])
