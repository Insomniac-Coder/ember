#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
# The total starts above 2⁶², so no block passes the test at its start: the
# loop runs checked and stops at the 808th addition.

fn main():
    xs: Array[int] = []
    for i in 0..1000:
        xs.push(1)
    total = 9223372036854775000
    for i in 0..1000:
        total = total + xs[i]
    println(total)
