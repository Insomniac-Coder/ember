#$ test: run-fail
#$ rules: STD-26, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow

fn main():
    low = -9223372036854775807 - 1
    println(abs(low))
