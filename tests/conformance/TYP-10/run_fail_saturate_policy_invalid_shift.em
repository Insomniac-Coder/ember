#$ test: run-fail
#$ rules: TYP-10, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `shift`

@overflow(saturate)
fn shifted(n: i8) -> i8:
    return 1i8 << n

fn main():
    println(shifted(8i8))
