#$ test: run-fail
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
# `[RNG-4]` — 200 turns adding 1 can take an `i8` total past 127: the bound
# does not fit the type, so the addition keeps its check and panics there.

fn main():
    total: i8 = 0
    for i in 0..200:
        total = total + 1
    println(total)
