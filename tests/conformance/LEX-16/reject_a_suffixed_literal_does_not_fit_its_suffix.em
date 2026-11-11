#$ test: compile-fail
#$ rules: LEX-16
# `[LEX-16]` — a literal with a suffix has the suffix's type, and one that does not fit it is
# `E2010`: `128i8` alone is refused, while `-128i8` is the negative constant of that type.

fn main():
    ok = -128i8
    a = 128i8                       #$ error[E2010]: the literal `128` does not fit in `i8`
    b = 256u8                       #$ error[E2010]: the literal `256` does not fit in `u8`
    c = -129i8                      #$ error[E2010]: the literal `-129` does not fit in `i8`
    println(ok)
    println(a)
    println(b)
    println(c)
