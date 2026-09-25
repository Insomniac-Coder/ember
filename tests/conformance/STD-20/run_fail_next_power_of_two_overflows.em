#$ test: run-fail
#$ rules: STD-20, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `next_power_of_two`
# ODR-039 — `next_power_of_two` past the type's `MAX` overflows, and overflow
# panics in every profile: 128 fits a `u8`, 256 does not.

fn next(b: u8) -> u8:
    return b.next_power_of_two()

fn main():
    println(next(100))
    println(next(200))
