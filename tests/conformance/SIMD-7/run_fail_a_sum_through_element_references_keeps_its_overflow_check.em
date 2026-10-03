#$ test: run-fail
#$ rules: SIMD-5, SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
#$ assert-c: contains(" >> 56ULL;")
# The second block fails its element-range proof and runs checked. Reading
# through the iteration's element reference must still report the overflowing
# addition, rather than wrapping or accepting the block's final total.

fn main():
    xs: Array[int] = []
    for i in 0..200:
        xs.push(1)
    xs[100] = 9223372036854775807
    total = 0
    for x in xs:
        total += x
    println(total)
