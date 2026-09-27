#$ test: run-pass
#$ rules: TYP-8, GRM-37, ATT-6
#$ profiles: debug, release, shipping
#$ stdout: -128
# A module attribute gives a function its lexical overflow policy.

#! module overflow(wrap)

fn add(a: i8, b: i8) -> i8:
    return a + b

fn main():
    println(add(127i8, 1i8))
