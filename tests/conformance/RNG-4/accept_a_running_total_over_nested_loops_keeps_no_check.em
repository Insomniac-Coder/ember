#$ test: run-pass
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 5994000
#$ assert-c-count: contains("ember_ck_") == 0
# `[RNG-4]` (D-530) — a running total changed in an inner loop changes on
# every inner turn: 1,000 runs of `total + xs[i]` (each element 0 to 6) on
# each of 2,000 outer turns, at most 12,000,000 in all, which an `int`
# holds, so no `+` keeps a check. The inner loop starts from the outer
# loop's bound.

fn main():
    xs: Array[int] = []
    for i in 0..1000:
        xs.push(i % 7)
    total = 0
    for round in 0..2000:
        for i in 0..1000:
            total = total + xs[i]
    println(total)
