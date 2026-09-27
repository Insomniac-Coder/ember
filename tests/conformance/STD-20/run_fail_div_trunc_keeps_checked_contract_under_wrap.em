#$ test: run-fail
#$ rules: STD-20, TYP-8, TYP-28
#$ profiles: debug, release, shipping
#$ panics: integer overflow

@overflow(wrap)
fn quotient(value: i8) -> i8:
    return value.div_trunc(-1i8)

fn main():
    println(quotient(i8.MIN))
