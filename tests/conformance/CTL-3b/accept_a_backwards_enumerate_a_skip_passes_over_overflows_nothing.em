#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release
#$ stdout: finished
# `[CTL-3b]`, `[STD-19]` (ODR-091; the owner's ruling of 2026-10-06) — a backwards `enumerate`
# numbers its greatest item first, and `skip(3)` leaves out all three items here: the loop gets no
# turn. The numbers, from `int.MAX - 1`, could pass `int`'s top, so they are `u64`s and leaving
# the items out overflows nothing, as through std's adapters (D-457): the program finishes.

fn main():
    ys = [10, 20, 30]
    for n, x in ys.enumerate(start=9223372036854775806).rev().skip(3):
        println(n, x)
    println("finished")
