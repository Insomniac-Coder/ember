#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c-count: contains("; ++_") == 3
# At iteration 20 the `+` makes `xs[i]` the least `int`, and the `-` that reads
# it back overflows: the group runs once for real, so the `-` reads the `+`'s
# result, not the older value (from which it would not overflow).

fn main():
    xs: Array[int] = []
    d: Array[int] = []
    for i in 0..40:
        xs.push(0)
        d.push(0)
    xs[20] = -9223372036854775807
    d[20] = -1
    for i in 0..40:
        xs[i] = xs[i] + d[i]
        xs[i] = xs[i] - 1
    println(xs[0])
