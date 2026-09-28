#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
# The total starts too close to the maximum for the proof: the loop keeps its
# checks and panics where it always did.

fn main():
    xs: Array[i32] = []
    for i in 0..100:
        xs.push(1000)
    total = 9223372036854775000
    for i in 0..100:
        total = total + (xs[i] as int)
    println(total)
