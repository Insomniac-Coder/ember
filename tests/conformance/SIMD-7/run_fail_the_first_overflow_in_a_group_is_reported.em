#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
# The `-` overflows at iteration 5 and the first `+` at iteration 9, in the
# same group: the panic is the `-`, the first overflow, as it always was.

fn main():
    big = 9223372036854775800
    a: Array[int] = []
    b: Array[int] = []
    out: Array[int] = []
    for i in 0..40:
        a.push(0)
        b.push(0)
        out.push(0)
    a[9] = big
    b[5] = -9223372036854775807 - 1
    for i in 0..40:
        out[i] = (a[i] + 100) - (b[i] + 100) + 0
    println(out[0])
