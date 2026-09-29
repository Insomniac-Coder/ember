#$ test: run-fail
#$ rules: CTL-3b, STD-19, TYP-8
#$ profiles: debug, release, shipping
#$ stdout:
#$ 9223372036854775801
#$ 9223372036854775804
#$ 9223372036854775807
#$ panics: integer overflow in `+`
# `[CTL-3b]`, `[STD-19]` — `enumerate(…).step_by(3)` numbers every item it
# pulls, including those `step_by` passes over; its last call, which finds the
# list at its end, pulls the eighth item, numbered past `int`'s top. The
# counted loop panics where the adapters would have: after the last turn.

fn main():
    xs: Array[int] = [0, 1, 2, 3, 4, 5, 6, 7]
    for (i, _) in xs.iter().enumerate(start=9223372036854775801).step_by(3):
        println(i)
