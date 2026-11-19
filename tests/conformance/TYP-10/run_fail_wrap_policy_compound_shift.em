#$ test: run-fail
#$ rules: TYP-10, TYP-8
#$ profiles: debug, development, release
#$ panics: `shift out of range`: `value <<= n` on line 9 shifts an `i8` by 8; use `wrapping_shl`

@overflow(wrap)
fn shifted(n: i8) -> i8:
    value: i8 = 1
    value <<= n
    return value

fn main():
    println(shifted(8i8))
