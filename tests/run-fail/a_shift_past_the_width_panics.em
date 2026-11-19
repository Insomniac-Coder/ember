#$ test: run-fail
#$ rules: TYP-10
#$ panics: `shift out of range`: `a << n` on line 7 shifts a `u32` by 32; use `wrapping_shl`
fn main():
    a: u32 = 1
    n: u32 = 32
    println(a << n)
