#$ test: run-pass
#$ rules: CT-7, CT-1, TYP-8, LEX-24
#$ profiles: debug, release, shipping
#$ stdout: -128 true

const LOW: i8 = -128i8 + 0i8
const WIDE: i128 = -170141183460469231731687303715884105728i128 + 0i128

fn main():
    println(LOW, WIDE == i128.MIN)
