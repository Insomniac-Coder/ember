#$ test: run-fail
#$ rules: STD-20, TYP-28
#$ profiles: debug, release, shipping
#$ panics: division by zero

@overflow(wrap)
fn remainder(value: i8, divisor: i8) -> i8:
    return value.rem_trunc(divisor)

fn main():
    println(remainder(i8.MIN, 0i8))
