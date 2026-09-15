#$ test: run-pass
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping
#$ stdout: 2

type Signed = i32 in -8 ..= 8
type Inverted = i32 in -4 ..= -4

fn negated(value: Signed) -> Signed:
    return -value

fn inverted() -> Inverted:
    return ~3

fn main():
    a: i32 = negated(Signed.clamped(-6))
    b: i32 = inverted()
    println(a + b)
