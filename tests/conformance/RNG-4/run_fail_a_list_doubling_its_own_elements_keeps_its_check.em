#$ test: run-fail
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `*`
# `[RNG-4]` — each element is stored as twice what it held, so no bound
# holds what the list can contain: the multiplication keeps its check and
# panics on the 63rd round.

fn main():
    xs: Array[int] = []
    for i in 0..4:
        xs.push(i + 1)
    for round in 0..70:
        for i in 0..xs.len():
            xs[i] = xs[i] * 2
    println(xs[0])
