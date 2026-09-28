#$ test: run-fail
#$ rules: OPT-2
#$ profiles: debug, release, shipping
#$ panics: index 3 is out of bounds for a length of 3
# The entry test fails (the last index, 4, is past the length), so the checked
# loop runs and panics where it always did: at the first bad index.

fn main():
    xs = [1, 2, 3]
    total = 0
    for i in 0..5:
        total = total + xs[i]
    println(total)
