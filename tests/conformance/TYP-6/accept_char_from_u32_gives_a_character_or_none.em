#$ test: run-pass
#$ rules: TYP-6
#$ profiles: debug, release
#$ stdout: Some('A') 233 128512
#$ stdout: None None None
# `[TYP-6]` — `char → u32` and `u8 → char` are casts; any other integer
# becomes a `char` only through `char.from_u32(x) -> Option[char]`, `None`
# when `x` is not a scalar value: a surrogate (`0xD800` to `0xDFFF`) or past
# `0x10FFFF` (D-469).

fn code(x: u32) -> u32:
    match char.from_u32(x):
        Some(c):
            return c as u32
        None:
            return 0

fn main():
    println(char.from_u32(65), code(233), code(0x1F600))
    println(char.from_u32(0xD800), char.from_u32(0xDFFF), char.from_u32(0x110000))
