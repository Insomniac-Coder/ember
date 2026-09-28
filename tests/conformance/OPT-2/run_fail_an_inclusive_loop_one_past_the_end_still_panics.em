#$ test: run-fail
#$ rules: OPT-2
#$ profiles: debug, release, shipping
#$ panics: index 3 is out of bounds for a length of 3
# `0..=3` reaches 3, one past the end: `a..=b`'s last index is `b` itself,
# so the entry test fails and the checked loop panics there.

fn main():
    xs = [1, 2, 3]
    total = 0
    for i in 0..=3:
        total = total + xs[i]
    println(total)
