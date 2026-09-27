#$ test: run-pass
#$ rules: CT-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 127 -128 0 127 0

#! module overflow(saturate)

const HIGH: i8 = 127i8 + 1i8
const LOW: i8 = -128i8 - 1i8
const ZERO: u8 = 0u8 - 1u8
const DIV: i8 = i8.MIN // -1i8
const REM: i8 = i8.MIN % -1i8

@overflow(wrap)
fn main():
    println(HIGH, LOW, ZERO, DIV, REM)
