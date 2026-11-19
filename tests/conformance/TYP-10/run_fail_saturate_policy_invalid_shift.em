#$ test: run-fail
#$ rules: TYP-10, TYP-8
#$ profiles: debug, development, release
#$ panics: `shift out of range`: `1i8 << n` on line 8 shifts an `i8` by 8; use `wrapping_shl`

@overflow(saturate)
fn shifted(n: i8) -> i8:
    return 1i8 << n

fn main():
    println(shifted(8i8))
