#$ test: run-fail
#$ rules: OPT-2, RNG-4
#$ profiles: debug, release, shipping
#$ panics: index 3 is out of bounds for a length of 3
# `[OPT-2]` — `i % 4` reaches 3 and the list has 3 elements: the entry test
# fails, the checked loop runs, and it panics where it always did.

fn main():
    xs: Array[int] = [10, 20, 30]
    total = 0
    for i in 0..8:
        total += xs[i % 4]
    println(total)
