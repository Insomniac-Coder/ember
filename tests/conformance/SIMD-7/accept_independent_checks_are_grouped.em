#$ test: run-pass
#$ rules: SIMD-7, SIMD-5, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 0 51 117
#$ assert-c-count: contains("((uint64_t)") == 2
# `out[i] = a[i] + b[i]` is in vectorisable form: its overflow checks are
# grouped. The grouped `+` computes its wrapped sum through `uint64_t` and its
# overflow from the signs, with no branch, once per iteration.

fn main():
    a: Array[int] = []
    b: Array[int] = []
    out: Array[int] = []
    for i in 0..40:
        a.push(i)
        b.push(2 * i)
        out.push(0)
    for i in 0..40:
        out[i] = a[i] + b[i]
    println(out[0], out[17], out[39])
