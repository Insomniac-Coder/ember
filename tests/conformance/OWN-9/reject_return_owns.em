#$ test: compile-fail
#$ rules: OWN-9, GRM-15
# `[OWN-9]` (verdict 148) — `return` hands the value over with no word, so `return owns xs` is
# `E0109`.

fn make() -> Array[int]:
    xs = [1, 2]
    return owns xs    #$ error[E0109]: `return` always hands the value over; remove `owns`: `return xs`

fn main():
    println(make())
