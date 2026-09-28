#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c-count: contains(" >> 63ULL)") == 2
# A `u64` `-` below zero at iteration 10, in the first group: the group reports it.

fn main():
    xs: Array[u64] = []
    ys: Array[u64] = []
    out: Array[u64] = []
    for i in 0..100:
        xs.push(5 as u64)
        ys.push(0 as u64)
        out.push(0 as u64)
    big: u64 = 5
    xs[10] = big
    ys[10] = 6 as u64
    for i in 0..100:
        out[i] = xs[i] - ys[i]
    println(out[0])
