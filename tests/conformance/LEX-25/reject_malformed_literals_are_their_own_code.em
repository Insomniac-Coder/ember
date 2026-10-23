#$ test: compile-fail
#$ rules: LEX-25, LEX-26, LEX-19, TXT-3
# ODR-186 — a literal the grammar does not write is `E0009`, an unclosed one `E0101`, a NUL in a
# C string literal `E5064` ([TXT-3]): each code is the rule's own ([DIA-6a]).

fn main():
    a = "C:\data"    #$ error[E0009]: unknown escape
    b = b"café"    #$ error[E0009]: a byte string may hold ASCII only
    c = f"a } b"    #$ error[E0009]: unmatched `}` in an f-string
    d = c"a\0b"    #$ error[E5064]: a C string literal may not contain a NUL byte
    println(a, b, c, d)
