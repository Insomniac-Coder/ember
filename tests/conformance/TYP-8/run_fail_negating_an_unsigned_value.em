#$ test: run-fail
#$ rules: TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ panics: integer overflow
# D-314 — negating an unsigned value overflows unless it is zero, and
# overflow panics in every profile; C wrapped it to `255`.

fn negate(x: u8) -> u8:
    return -x

fn main():
    println(negate(0))
    println(negate(1))
