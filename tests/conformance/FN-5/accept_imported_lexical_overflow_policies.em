#$ test: run-pass
#$ rules: FN-5, TYP-8, TYP-18, CLO-1, MOD-3
#$ profiles: debug, development, release
#$ stdout: -128 -128 -128 127

from support.overflow_policy import defaulted, generic, callback

@overflow(saturate)
fn main():
    step = callback()
    println(defaulted(127i8), generic[int](0, 127i8), step(127i8),
        defaulted(127i8, 127i8 + 1i8))
