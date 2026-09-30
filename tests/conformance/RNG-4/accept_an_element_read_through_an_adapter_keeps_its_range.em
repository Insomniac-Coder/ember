#$ test: run-pass
#$ rules: RNG-4, CTL-3b
#$ profiles: debug, release
#$ stdout: 14
#$ assert-c-count: contains("ember_ck_") == 0
# `[RNG-4]` — what a list's elements can hold reaches an element read through
# an adapter's loop: the element reference the loop binds, written by each of
# a `chain`'s loops from a different list and copied into the loop's name,
# holds an element of one of those lists (ADR-083). So `x + round` and
# `y * 3 + i`, which cannot overflow, keep no check.

fn main():
    xs: Array[int] = []
    ys: Array[int] = []
    for i in 0..1000:
        xs.push(i % 100)
        ys.push(i % 7)
    total = 0
    for round in 0..10:
        for x in xs.iter().chain(ys.iter()):
            total ^= x + round
        for (i, y) in ys.iter().enumerate():
            total ^= y * 3 + i
    println(total)
