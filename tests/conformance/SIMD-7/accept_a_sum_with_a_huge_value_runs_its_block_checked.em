#$ test: run-pass
#$ rules: SIMD-7, SIMD-5, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 19680
#$ assert-c: contains(" >> 56ULL;")
# Two values are too large for the run-time proof (2⁶⁰ and −2⁶⁰), though the
# total never overflows: the block holding the first fails the test and it
# and the rest of the loop run checked, giving the exact total.

fn main():
    xs: Array[int] = []
    for i in 0..200:
        xs.push(i)
    xs[70] = 1152921504606846976
    xs[150] = -1152921504606846976
    total = 0
    for i in 0..200:
        total = total + xs[i]
    println(total)
