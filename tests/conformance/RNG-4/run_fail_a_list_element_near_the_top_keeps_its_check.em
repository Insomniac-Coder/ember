#$ test: run-fail
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
# `[RNG-4]` — the list's elements hold what was stored in them, and one
# stored value is near `int`'s top: adding it to the total keeps its check.

fn main():
    xs: Array[int] = []
    for i in 0..4:
        xs.push(i)
    xs.push(9223372036854775000)
    total = 0
    for x in xs:
        total = total + x + 1000
    println(total)
