#$ test: run-fail
#$ rules: TYP-8, TYP-28
#$ profiles: debug, release, shipping
#$ panics: division by zero

@overflow(saturate)
fn divide(divisor: i8) -> i8:
    return 7i8 // divisor

fn main():
    println(divide(0i8))
