#$ test: run-fail
#$ rules: FN-5, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow

@overflow(panic)
fn defaulted(x: i8, value: i8 = x + 1i8) -> i8:
    return value

@overflow(wrap)
fn main():
    println(defaulted(127i8))
