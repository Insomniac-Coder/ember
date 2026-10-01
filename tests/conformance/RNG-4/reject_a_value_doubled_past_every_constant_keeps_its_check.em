#$ test: run-fail
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release
#$ stdout: 4611686018427387904
#$ panics: integer overflow in `*`
# `[RNG-4]` — widening tries the constants the loop holds (here around 2 and
# 70) before `int`'s end, but `big` doubles past every one of them, so it is
# widened to the end and `big * 2` keeps its check: the 63rd doubling
# overflows.

fn main():
    big = 1
    for i in 0..70:
        if i == 62:
            println(big)
        big = big * 2
    println(big)
