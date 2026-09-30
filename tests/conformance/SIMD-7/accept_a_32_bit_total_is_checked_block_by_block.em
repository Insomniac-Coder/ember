#$ test: run-pass
#$ rules: SIMD-7, SIMD-5, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 299700
#$ assert-c: contains(" >> 24ULL;")
# An `i32` total of `i32` values: the widths prove nothing, so the run-time
# test takes values within ±2²³ and the total within ±2³⁰.

fn main():
    xs: Array[i32] = []
    for i in 0..1000:
        xs.push((i % 7) as i32)
    total: i32 = 0
    for round in 0..100:
        for i in 0..1000:
            total = total + xs[i]
    println(total)
