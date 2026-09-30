#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c: contains(" >> 56ULL;")
# `total = total - x` is a running total too: at iteration 60 it goes below
# the least `int`, and the checked block stops there.

fn main():
    xs: Array[int] = []
    for i in 0..200:
        xs.push(1)
    xs[60] = 9223372036854775807
    total = 0
    for i in 0..200:
        total = total - xs[i]
    println(total)
