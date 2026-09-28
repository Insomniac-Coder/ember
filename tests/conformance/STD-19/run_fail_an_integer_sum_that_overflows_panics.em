#$ test: run-fail
#$ rules: STD-19, STD-5, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
# `[STD-5]` — an integer sum panics on overflow, as `+` does.

fn main():
    big: Array[int] = [9223372036854775807, 1]
    println(big.iter().sum())
