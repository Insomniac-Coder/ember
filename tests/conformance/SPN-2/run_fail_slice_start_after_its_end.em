#$ test: run-fail
#$ rules: SPN-2
#$ profiles: debug, release, shipping
#$ panics: index 2 is out of bounds for a length of 1
# A slice that starts after it ends panics: the start is checked against the
# end.

fn main():
    xs = [1, 2, 3]
    println(xs[2..1])
