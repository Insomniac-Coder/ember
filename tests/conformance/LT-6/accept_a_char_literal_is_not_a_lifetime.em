#$ test: run-pass
#$ rules: LT-6, LEX-22
# The other side of `[LEX-22]`'s disambiguation: a `'` followed by an
# identifier that *is* closed by a second `'` is a char literal, and only an
# unclosed one is a `Lifetime` token. If that went wrong, every `'x'` in the
# language would become a reserved-feature error.

fn main():
    c: char = 'a'
    println(c)
#$ stdout: a
