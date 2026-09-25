#$ test: run-fail
#$ rules: STD-15, SPN-4
#$ panics: index 0 is out of bounds for a length of 0
# `[STD-15]` (ODR-031) — `windows(0)` panics, as `chunks(0)` does (`[SPN-4]`).

fn main():
    xs = [1, 2, 3]
    for w in xs.windows(0):
        println(w.len())
