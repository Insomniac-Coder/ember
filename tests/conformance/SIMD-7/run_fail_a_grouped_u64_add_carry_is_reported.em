#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
#$ assert-c-count: contains(" >> 63ULL)") == 2
# A `u64` `+` past its greatest value at iteration 80, in the third group: the group reports it.

fn main():
    xs: Array[u64] = []
    ys: Array[u64] = []
    out: Array[u64] = []
    for i in 0..100:
        xs.push(1 as u64)
        ys.push(1 as u64)
        out.push(0 as u64)
    big: u64 = 18446744073709551600
    xs[80] = big
    ys[80] = 100 as u64
    for i in 0..100:
        out[i] = xs[i] + ys[i]
    println(out[0])
