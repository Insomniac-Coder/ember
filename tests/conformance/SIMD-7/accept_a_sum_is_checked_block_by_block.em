#$ test: run-pass
#$ rules: SIMD-7, SIMD-5, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 5994000
#$ assert-c: contains(" >> 56ULL;")
# A running total of `int`s that the widths cannot prove safe (ODR-086): in
# blocks of 64 iterations the total is tested small enough at the block's
# start (`>> 63`) and every value added small enough (`>> 56`), and the block
# adds with no check on each operation. 1,000 values: 15 blocks, then 40
# checked one at a time.

# Lent to `unknown`, the list holds values the range facts cannot know
# (`[RNG-4]`): known (0 to 6), they prove the total small enough, and it
# keeps no check at all (ADR-142, D-530).
fn unknown(mut xs: Array[int]):
    pass

fn main():
    xs: Array[int] = []
    for i in 0..1000:
        xs.push(i % 7)
    unknown(xs)
    total = 0
    for round in 0..2000:
        for i in 0..1000:
            total = total + xs[i]
    println(total)
