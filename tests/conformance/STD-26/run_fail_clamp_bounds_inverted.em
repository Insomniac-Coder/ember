#$ test: run-fail
#$ rules: STD-26, ERR-13
#$ profiles: debug, release, shipping
#$ panics: clamp: the lower bound is greater than the upper bound

fn main():
    println(clamp(5, 10, 0))
