#$ test: run-fail
#$ rules: TYP-10, TYP-8
#$ profiles: debug, development, release
#$ panics: integer overflow in `shift`

@overflow(wrap)
fn shifted(n: i8) -> i8:
    return 1i8 << n

fn main():
    println(shifted(-1i8))
