#$ test: run-fail
#$ rules: TYP-28
#$ profiles: debug, release, shipping
#$ panics: division by zero
# A zero divisor panics for `//` as for `%`.

fn main():
    zero = 0
    println(5 // zero)
