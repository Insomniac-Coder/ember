#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
#$ assert-c: contains(" >> 56ULL)")
# The 101st addition overflows, inside the second block: the block fails the
# test and runs checked, which stops at that addition.

fn main():
    xs: Array[int] = []
    for i in 0..200:
        xs.push(1)
    xs[100] = 9223372036854775807
    total = 0
    for i in 0..200:
        total = total + xs[i]
    println(total)
