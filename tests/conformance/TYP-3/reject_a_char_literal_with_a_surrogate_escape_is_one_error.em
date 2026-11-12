#$ test: compile-fail
#$ rules: TYP-3, LEX-26, DIA-26
# `[TYP-3]` — no escape names a surrogate, so no `char` literal is one: `'\u{D800}'` is `E0009`, on
# the escape. `[DIA-26]`: it is one mistake, so one error, not one for the escape and one more for
# the literal that holds it.

fn main():
    c = '\u{D800}'    #$ error[E0009]: 0xd800 is not a Unicode scalar value
    println(c)
