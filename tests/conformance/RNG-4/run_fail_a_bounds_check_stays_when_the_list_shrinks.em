#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, release, shipping
#$ panics: index 0 is out of bounds for a length of 0
# `[RNG-4]` — the loop's test put `i` below the length, but `clear()` changes
# the length: the fact is forgotten and the index stays checked.

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    i = 0
    total = 0
    while i < xs.len():
        xs.clear()
        total += xs[i]
        i += 1
    println(total)
