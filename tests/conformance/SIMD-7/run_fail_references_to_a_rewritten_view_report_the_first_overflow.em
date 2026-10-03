#$ test: run-fail
#$ rules: SIMD-5, SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c: !contains(" << ")
#$ assert-c-count: contains("; ++_") == 3
# The second operation overflows at iteration 5, before the first operation
# overflows at iteration 9. Saving and rerunning a group must account for
# shared references as reads of the view, and restore its original values.

fn main():
    xs: Array[int] = []
    d: Array[int] = []
    e: Array[int] = []
    for i in 0..40:
        xs.push(0)
        d.push(0)
        e.push(0)
    xs[5] = -9223372036854775807 - 1
    e[5] = 1
    xs[9] = 9223372036854775807
    d[9] = 1
    for i in 0..40:
        before: ref int = ref xs[i]
        next = before + d[i]
        xs[i] = next
        after: ref int = ref xs[i]
        last = after - e[i]
        xs[i] = last
    println(xs[0])
