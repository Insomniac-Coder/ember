#$ test: run-pass
#$ rules: CT-4, TYP-8, MOD-3
#$ profiles: debug, release, shipping
#$ stdout: -128 127 255

from support.wrapped_consts import HIGH, LOW, ZERO

@overflow(saturate)
fn main():
    println(HIGH, LOW, ZERO)
