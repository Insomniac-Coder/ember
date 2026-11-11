#$ test: compile-fail
#$ rules: LEX-22
# `[LEX-22]` — Ember has no lifetime syntax: a `'` begins a character literal and nothing else; an
# unterminated one is `E0008`.

fn main():
    c = 'a                          #$ error[E0008]: unterminated character literal
    println(c)
