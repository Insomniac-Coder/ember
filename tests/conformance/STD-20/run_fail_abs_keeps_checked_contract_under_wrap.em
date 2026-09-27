#$ test: run-fail
#$ rules: STD-20, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`

@overflow(wrap)
fn magnitude(value: i8) -> i8:
    return value.abs()

fn main():
    println(magnitude(i8.MIN))
