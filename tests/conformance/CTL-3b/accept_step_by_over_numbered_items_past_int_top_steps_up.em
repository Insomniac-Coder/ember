#$ test: run-pass
#$ rules: CTL-3b, STD-19, TYP-8
#$ profiles: debug, release, shipping
#$ stdout:
#$ 9223372036854775801
#$ 9223372036854775804
#$ 9223372036854775807
#$ finished
# `[CTL-3b]`, `[STD-19]` — `enumerate(…).step_by(3)` numbers every item it pulls, including those
# `step_by` passes over; its last call pulls the eighth item, numbered past `int`'s top. The
# numbers are `u64`s, which hold it (the owner's ruling of 2026-10-06): nothing overflows.

fn main():
    xs: Array[int] = [0, 1, 2, 3, 4, 5, 6, 7]
    for (i, _) in xs.iter().enumerate(start=9223372036854775801).step_by(3):
        println(i)
    println("finished")
