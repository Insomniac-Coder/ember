#$ test: run-pass
#$ rules: TYP-6
#$ stdout: 233 A
# `[TYP-6]` — `char → u32` is the scalar value and `u8 → char` is always a
# scalar value; other integers become a `char` only through `char.from_u32`.

fn main():
    c = 'é'
    b: u8 = 65
    println(c as u32, b as char)
