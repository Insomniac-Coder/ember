#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
#$ assert-c: contains(" >> 24ULL;")
# An `i32` total overflows at iteration 50, inside the first block.

fn main():
    xs: Array[i32] = []
    for i in 0..200:
        xs.push(1 as i32)
    xs[50] = 2147483647 as i32
    total: i32 = 0
    for i in 0..200:
        total = total + xs[i]
    println(total)
