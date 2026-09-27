#$ test: run-fail
#$ rules: STD-20, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `*`

@overflow(saturate)
fn power(value: i8) -> i8:
    return value.pow(2)

fn main():
    println(power(12i8))
