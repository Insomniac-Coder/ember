#$ test: run-pass
#$ rules: LEX-22
#$ stdout: a
# `[LEX-22]` — a `'` begins a character literal and nothing else.

fn main():
    c = 'a'
    println(c)
