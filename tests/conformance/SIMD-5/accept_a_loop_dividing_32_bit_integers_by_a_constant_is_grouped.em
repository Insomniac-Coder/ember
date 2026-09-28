#$ test: run-pass
#$ rules: SIMD-5, SIMD-7
#$ profiles: debug, release, shipping
#$ stdout: 49995000 4 7
#$ assert-c-count: contains(">= 32ULL") == 1
# `[SIMD-5]` (ODR-088) — a 32-bit division by a constant becomes a vector
# multiply and shift, so a loop doing it stays in vectorisable form and its
# overflow check is grouped (`[SIMD-7]`), 32 iterations at a time.

# Lent to a function, a list holds values the range facts cannot know
# (`[RNG-4]`), so the checks this test looks at stay.
fn unknown[T](mut xs: Array[T]):
    pass

fn main():
    xs: Array[i32] = []
    for i in 0..10000:
        xs.push(i as i32)
    unknown(xs)
    ys: Array[i32] = []
    for i in 0..10000:
        ys.push(0)
    for i in 0..10000:
        ys[i] = xs[i] * 3 // 7
    total: i64 = 0
    for i in 0..10000:
        total = total + (xs[i] as i64)
    println(total, ys[10], ys[17])
