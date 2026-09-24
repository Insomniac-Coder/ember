#$ test: run-pass
#$ rules: LT-6, LEX-22
# `[LEX-22]` — a `'` followed by an identifier and closed by a second `'` is
# a character literal: there is no lifetime syntax to confuse it with.

fn main():
    c: char = 'a'
    println(c)
#$ stdout: a
