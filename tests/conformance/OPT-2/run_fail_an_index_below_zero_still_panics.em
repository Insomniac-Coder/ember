#$ test: run-fail
#$ rules: OPT-2
#$ profiles: debug, release, shipping
#$ panics: index -1 is out of bounds for a length of 3
# `i - 1` is -1 on the first pass, so the entry test fails and the checked
# loop panics on the index as it always did.

fn main():
    xs = [1, 2, 3]
    total = 0
    for i in 0..3:
        total = total + xs[i - 1]
    println(total)
