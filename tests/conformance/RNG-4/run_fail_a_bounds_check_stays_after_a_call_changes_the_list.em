#$ test: run-fail
#$ rules: RNG-4, FN-9
#$ profiles: debug, release, shipping
#$ panics: index 1 is out of bounds for a length of 0
# `[RNG-4]` — `k < xs.len()` held before the call, but `shrink` is lent the
# list and empties it: the length is forgotten at the call.

fn shrink(mut xs: Array[int]):
    xs.clear()

fn main():
    xs: Array[int] = [1, 2, 3]
    k = 1
    if k < xs.len():
        shrink(xs)
        println(xs[k])
