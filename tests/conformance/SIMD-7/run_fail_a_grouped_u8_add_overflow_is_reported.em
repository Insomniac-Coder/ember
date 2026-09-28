#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
#$ assert-c-count: contains(" >> 63ULL)") == 2
# A `u8` `+` past 255 at iteration 40, in the second group: the group reports it.

fn main():
    xs: Array[u8] = []
    ys: Array[u8] = []
    out: Array[u8] = []
    for i in 0..100:
        xs.push(200 as u8)
        ys.push(0 as u8)
        out.push(0 as u8)
    big: u8 = 200
    xs[40] = big
    ys[40] = 100 as u8
    for i in 0..100:
        out[i] = xs[i] + ys[i]
    println(out[0])
