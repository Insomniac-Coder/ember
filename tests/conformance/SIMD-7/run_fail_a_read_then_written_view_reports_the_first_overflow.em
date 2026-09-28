#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
#$ assert-c: !contains(" << ")
#$ assert-c-count: contains("for (;") == 3
# `xs[i]` is read and then written, and the loop has one check. Each group
# runs once in wrapping arithmetic, storing as it goes, and a set overflow flag
# panics at the group's end with that check's location: the first overflow's,
# whatever iteration it was. The stores after it are never read (the process
# aborts). The group is a C `for` loop, like both copies of the loop.

fn main():
    xs: Array[int] = []
    d: Array[int] = []
    for i in 0..40:
        xs.push(i)
        d.push(1)
    xs[20] = 9223372036854775807
    for i in 0..40:
        xs[i] = xs[i] + d[i]
    println(xs[0])
