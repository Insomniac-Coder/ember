#$ test: run-pass
#$ rules: RNG-4, CTL-3b, STD-26
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_panic_bounds(") == 0
#$ stdout: 22 30 15
# `[RNG-4]` — in a stepped loop over `start..stop` by a positive `step`, each
# value lies between `start` and `stop - 1`: it is computed from an index
# below the loop's count of values. With `stop` a list's length, indexing the
# list needs no check, for `step_by` and for `range(start, stop, step)`.

fn main():
    xs: Array[int] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    a = 0
    for i in (0..xs.len()).step_by(3):
        a += xs[i]
    b = 0
    for i in range(1, xs.len(), 2):
        b += xs[i]
    c = 0
    for i in range(4, len(xs), 5):
        c += xs[i]
    println(a, b, c)
