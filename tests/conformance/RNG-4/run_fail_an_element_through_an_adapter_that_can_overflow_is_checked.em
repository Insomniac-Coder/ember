#$ test: run-fail
#$ rules: RNG-4, CTL-3b
#$ profiles: debug, release
#$ stdout:
#$ 5
#$ 11
#$ panics: integer overflow in `+`
# `[RNG-4]` — an element read through a `chain`'s loop keeps its check when
# one of the lists it may come from holds a value the addition overflows: the
# range is the hull of every list the element reference reaches (ADR-083).

fn main():
    xs: Array[int] = [1, 2]
    ys: Array[int] = [int.MAX]
    total = 0
    for x in xs.iter().chain(ys.iter()):
        total += x + 4
        println(total)
