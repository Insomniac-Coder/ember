#$ test: run-pass
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping
#$ stdout: 4
#$ 2

type Wide = i32 in -100 ..= 100
type Narrow = i32 in -10 ..= 10
type Residue = i32 in -9 ..= 9

fn quotient(value: Wide) -> Narrow:
    return value // 10

fn remainder(value: Wide) -> Residue:
    return value % 10

fn main():
    integer: i32 = quotient(42)
    residue: i32 = remainder(42)
    println(integer)
    println(residue)
