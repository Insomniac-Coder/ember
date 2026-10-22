#$ test: run-pass
#$ rules: LEX-26
#$ stdout: eight
#$ é
#$ {x} 3
# `[LEX-26]` — a literal written as the grammar gives it: a raw string's prefix of eight `#`, a
# character literal of one Unicode scalar value, an f-string's `}` doubled outside a hole.

fn main():
    a = r########"eight"########
    println(a)
    c = 'é'
    println(c)
    n = 3
    println(f"{{x}} {n}")
