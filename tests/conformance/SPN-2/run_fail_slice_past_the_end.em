#$ test: run-fail
#$ rules: SPN-2
#$ profiles: debug, release, shipping
#$ panics: index 5 is out of bounds for a length of 3
# A slice's end past the length panics in every profile.

fn main():
    xs = [1, 2, 3]
    println(xs[2..5])
