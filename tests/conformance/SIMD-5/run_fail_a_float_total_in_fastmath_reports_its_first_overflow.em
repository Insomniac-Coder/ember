#$ test: run-fail
#$ rules: SIMD-5, SIMD-7, TYP-9
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c-count: contains(">> EMBER_OVERFLOW_SHIFT;") == 2
# A `@fastmath` loop keeping a float total is in vectorisable form, so its
# checks are grouped (`[SIMD-7]`, the flag's shift below); the `-` overflows at iteration 5 and the
# first `+` at iteration 9, in one group, and the panic is still the `-`,
# the first overflow.

@fastmath
fn run() -> f64:
    big = 9223372036854775800
    a: Array[int] = []
    b: Array[int] = []
    out: Array[int] = []
    fs: Array[f64] = []
    for i in 0..40:
        a.push(0)
        b.push(0)
        out.push(0)
        fs.push(0.5)
    a[9] = big
    b[5] = -9223372036854775807 - 1
    total = 0.0
    for i in 0..40:
        out[i] = (a[i] + 100) - (b[i] + 100) + 0
        total += fs[i]
    return total + out[0] as f64

fn main():
    println(run())
