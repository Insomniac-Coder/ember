#$ test: run-pass
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping
#$ stdout: 4
#$ 4.2
#$ 2

type Wide = i32 in -100 ..= 100
type Narrow = i32 in -10 ..= 10
type Residue = i32 in -9 ..= 9
type WideRatio = f32 in -100.0 ..= 100.0
type NarrowRatio = f32 in -10.0 ..= 10.0

fn quotient(value: Wide) -> Narrow:
    return value // 10

fn quotient_ratio(value: WideRatio) -> NarrowRatio:
    return value / 10.0

fn remainder(value: Wide) -> Residue:
    return value % 10

fn main():
    integer: i32 = quotient(42)
    ratio: f32 = quotient_ratio(42.0)
    residue: i32 = remainder(42)
    println(integer)
    println(ratio)
    println(residue)
