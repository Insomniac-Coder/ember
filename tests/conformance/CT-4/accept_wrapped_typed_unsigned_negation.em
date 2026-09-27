#$ test: run-pass
#$ rules: CT-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 255

#! module overflow(wrap)

const NEGATED: u8 = -1u8

fn main():
    println(NEGATED)
