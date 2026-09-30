#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c-count: contains(" >> 63ULL;") == 2
# An `i32` `-` below its least value at iteration 50, in the second group: the group reports it.

fn main():
    xs: Array[i32] = []
    ys: Array[i32] = []
    out: Array[i32] = []
    for i in 0..100:
        xs.push(0 as i32)
        ys.push(0 as i32)
        out.push(0 as i32)
    big: i32 = -2147483600
    xs[50] = big
    ys[50] = 100 as i32
    for i in 0..100:
        out[i] = xs[i] - ys[i]
    println(out[0])
