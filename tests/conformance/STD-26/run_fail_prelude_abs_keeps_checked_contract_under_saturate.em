#$ test: run-fail
#$ rules: STD-26, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow

@overflow(saturate)
fn magnitude(value: i8) -> i8:
    return abs(value)

fn main():
    println(magnitude(i8.MIN))
