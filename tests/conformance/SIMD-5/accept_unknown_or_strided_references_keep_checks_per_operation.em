#$ test: run-pass
#$ rules: SIMD-5, SIMD-7
#$ profiles: debug, release, shipping
#$ stdout: 240 90
#$ assert-c: !contains(" >> 56ULL;")
# A reference passed into the function is not a unit-stride element access,
# and xs[2 * i] is strided. Neither gains a grouped running-total proof.

fn repeated(xs: Span[int], value: ref int) -> int:
    total = 0
    for _ in xs:
        total += value
    return total

fn strided(xs: Span[int]) -> int:
    total = 0
    for i in 0..xs.len() // 2:
        value: ref int = ref xs[2 * i]
        total += value
    return total

fn main():
    xs: Array[int] = []
    for i in 0..20:
        xs.push(i)
    value = 12
    println(repeated(xs.as_span(), ref value), strided(xs.as_span()))
