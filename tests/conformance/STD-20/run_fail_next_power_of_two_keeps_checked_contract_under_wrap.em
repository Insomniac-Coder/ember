#$ test: run-fail
#$ rules: STD-20, TYP-8
#$ profiles: debug, development, release
#$ panics: integer overflow in `next_power_of_two`

@overflow(wrap)
fn next(value: u8) -> u8:
    return value.next_power_of_two()

fn main():
    println(next(200u8))
