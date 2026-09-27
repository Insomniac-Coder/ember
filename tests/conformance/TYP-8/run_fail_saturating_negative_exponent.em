#$ test: run-fail
#$ rules: TYP-8, TYP-30
#$ profiles: debug, release, shipping
#$ panics: `**` with a negative exponent

@overflow(saturate)
fn power(exponent: i8) -> i8:
    return 2i8 ** exponent

fn main():
    println(power(-1i8))
