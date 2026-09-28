#$ test: run-fail
#$ rules: RNG-4, CTL-3b
#$ profiles: debug, release, shipping
#$ panics: index 12 is out of bounds for a length of 10
# `[RNG-4]` — the loop runs to 13, past the list's length, so indexing it
# keeps its check and panics at the first value past the end.

fn main():
    xs: Array[int] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    total = 0
    for i in (0..13).step_by(4):
        total += xs[i]
    println(total)
