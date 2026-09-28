#$ test: run-fail
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
# `[RNG-4]` — the list is lent to a function that writes an element the
# caller cannot see, so what its elements hold is not known and the sum keeps
# its check.

fn put_big(mut xs: Array[int]):
    xs[0] = 9223372036854775807

fn main():
    xs: Array[int] = []
    for i in 0..10:
        xs.push(i % 7)
    put_big(xs)
    total = 0
    for i in 0..xs.len():
        total = total + xs[i]
    println(total)
