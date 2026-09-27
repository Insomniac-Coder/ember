#$ test: run-pass
#$ rules: STD-20, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: -128 127

@overflow(wrap)
fn wrapped_argument() -> i8:
    return (127i8 + 1i8).pow(1)

@overflow(saturate)
fn saturated_argument() -> i8:
    return (127i8 + 1i8).pow(1)

fn main():
    println(wrapped_argument(), saturated_argument())
