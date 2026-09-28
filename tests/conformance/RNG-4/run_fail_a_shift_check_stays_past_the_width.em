#$ test: run-fail
#$ rules: RNG-4, TYP-10
#$ profiles: debug, release, shipping
#$ stdout: 8
#$ panics: integer overflow in `shift`
# `[RNG-4]` — `k % 64` is below 64, but the shifted value is an `i32`: an
# amount of 40 is past its width and still panics.

fn bit(k: int) -> i32:
    s = k % 64
    return (1 as i32) << (s as i32)

fn main():
    println(bit(3))
    println(bit(40))
