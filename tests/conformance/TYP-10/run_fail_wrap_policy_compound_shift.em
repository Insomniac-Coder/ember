#$ test: run-fail
#$ rules: TYP-10, TYP-8
#$ profiles: debug, development, release
#$ panics: integer overflow in `shift`

@overflow(wrap)
fn shifted(n: i8) -> i8:
    value: i8 = 1
    value <<= n
    return value

fn main():
    println(shifted(8i8))
