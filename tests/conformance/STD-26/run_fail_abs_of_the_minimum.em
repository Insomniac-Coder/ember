#$ test: run-fail
#$ rules: STD-26, TYP-8
#$ profiles: debug, development, release
#$ panics: integer overflow

fn main():
    low = -9223372036854775807 - 1
    println(abs(low))
