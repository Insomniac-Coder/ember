#$ test: run-fail
#$ rules: TYP-8, TYP-1
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
# D-272 — `u128` subtraction below zero overflows and panics.

fn less(a: u128, b: u128) -> u128:
    return a - b

fn main():
    println(less(2, 1))
    println(less(1, 2))
