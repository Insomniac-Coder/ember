#$ test: run-fail
#$ rules: STD-15, ERR-13
#$ profiles: debug, release, shipping
#$ panics: remove index out of range

fn main():
    xs = [1, 2]
    println(xs.remove(2))
