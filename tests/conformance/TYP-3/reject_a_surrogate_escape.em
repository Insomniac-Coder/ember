#$ test: compile-fail
#$ rules: TYP-3
# `[TYP-3]` — no escape names a surrogate, so no literal makes a `char` outside the Unicode scalar
# values.

fn main():
    s = "\u{DFFF}"    #$ error[E0009]: is not a Unicode scalar value
    println(s)
