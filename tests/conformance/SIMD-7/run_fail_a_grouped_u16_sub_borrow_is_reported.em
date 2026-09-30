#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c-count: contains(" >> 63ULL;") == 2
# A `u16` `-` below zero at iteration 70, in the third group: the group reports it.

fn main():
    xs: Array[u16] = []
    ys: Array[u16] = []
    out: Array[u16] = []
    for i in 0..100:
        xs.push(5 as u16)
        ys.push(0 as u16)
        out.push(0 as u16)
    big: u16 = 5
    xs[70] = big
    ys[70] = 6 as u16
    for i in 0..100:
        out[i] = xs[i] - ys[i]
    println(out[0])
