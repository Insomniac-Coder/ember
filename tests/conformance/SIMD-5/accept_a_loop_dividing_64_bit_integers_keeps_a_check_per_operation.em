#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 1666616667
#$ assert-c-count: contains("EMBER_OVERFLOW_SHIFT") == 0
#$ assert-c-count: contains(">= 32ULL") == 0
#$ assert-c-count: contains(">= 16ULL") == 0
#$ assert-c-count: contains("ember_ck_add_i64(") == 1
# `[SIMD-5]` (ODR-088) — no vector instruction set divides 64-bit integers by
# a constant that is not a power of two, so this loop runs one iteration at a
# time and is not in vectorisable form: its sum keeps one check per addition,
# with no grouped copy (which would only add work here).

fn main():
    total = 0
    for i in 0..100000:
        total = total + i // 3
    println(total)
