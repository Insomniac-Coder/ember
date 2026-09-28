#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 16661667
#$ assert-c-count: contains("EMBER_OVERFLOW_SHIFT") == 0
#$ assert-c-count: contains(">= 32ULL") == 0
#$ assert-c-count: contains(">= 16ULL") == 0
#$ assert-c-count: contains("ember_ck_add_i64(") == 1
# `[SIMD-5]` (ODR-088) — no vector instruction set divides 64-bit integers by
# a constant that is not a power of two, so this loop runs one iteration at a
# time and is not in vectorisable form: its sum, of values nothing bounds,
# keeps one check per addition, with no grouped copy (which would only add
# work here).

fn total_of_thirds(xs: Array[int]) -> int:
    total = 0
    for i in 0..xs.len():
        total = total + xs[i] // 3
    return total

fn main():
    xs: Array[int] = []
    for i in 0..10000:
        xs.push(i)
    println(total_of_thirds(xs))
