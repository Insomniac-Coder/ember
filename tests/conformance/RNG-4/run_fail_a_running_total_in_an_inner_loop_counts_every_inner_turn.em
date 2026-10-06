#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, release, shipping
#$ panics: integer overflow
# `[RNG-4]` (D-530) — a running total changed inside an inner loop changes on
# every inner turn: one outer turn adds 1,000,000, not 1. The outer loop's
# bound once took the inner `+` as running once a turn (at most 5 here), so
# `total * 1000` lost its check and wrapped at 3,000,000 instead of
# panicking.

fn main():
    total: i32 = 0
    y: i32 = 0
    for round in 0..5:
        y = total * 1000
        for i in 0..1000000:
            total = total + 1
    println(y, total)
