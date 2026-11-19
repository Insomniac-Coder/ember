#$ test: run-fail
#$ rules: TYP-10, TYP-8
#$ profiles: debug, development, release
#$ panics: `shift out of range`: `1i8 << n` on line 8 shifts an `i8` by -1; use `wrapping_shl`

@overflow(wrap)
fn shifted(n: i8) -> i8:
    return 1i8 << n

fn main():
    println(shifted(-1i8))
