#$ test: run-pass
#$ rules: RNG-4, RNG-10, TYP-10
#$ profiles: debug, release, shipping
#$ stdout: 63

type Positive = i32 in 0 ..= 100
type Shifted = i32 in 0 ..= 200
type Signed = i32 in -100 ..= 100
type NarrowSigned = i32 in -50 ..= 50

fn left(value: Positive) -> Shifted:
    return value << 1

fn right(value: Signed) -> NarrowSigned:
    return value >> 1

fn main():
    shifted: i32 = left(42)
    narrowed: i32 = right(Signed.clamped(-42))
    println(shifted + narrowed)
