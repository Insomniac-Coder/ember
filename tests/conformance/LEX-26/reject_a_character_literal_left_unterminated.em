#$ test: compile-fail
#$ rules: LEX-26, LEX-22
# `[LEX-26]`, `[LEX-22]` — an unterminated character literal is `E0008`.

fn main():
    c = 'a    #$ error[E0008]: unterminated character literal
    println(1)
