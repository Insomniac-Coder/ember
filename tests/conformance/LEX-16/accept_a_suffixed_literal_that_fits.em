#$ test: run-pass
#$ rules: LEX-16
#$ stdout: -128
#$ 127
#$ 255
# `[LEX-16]` — a suffixed literal that fits its suffix's type is accepted, and a minus directly
# before it forms the negative constant of that type.

fn main():
    println(-128i8)
    println(127i8)
    println(255u8)
