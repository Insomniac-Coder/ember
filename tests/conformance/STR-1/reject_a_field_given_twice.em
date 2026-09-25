#$ test: compile-fail
#$ rules: STR-1, TYP-25
# `[TYP-25]` — each field takes one argument: a positional argument and a
# named one for the same field, or the same name twice, is `E1030`; a
# positional argument after a named one is `E2020`.

struct P:
    a: int
    b: int

fn main():
    p = P(1, a = 2, b = 3)  #$ error[E1030]: field `a` is given twice
    q = P(a = 1, b = 2, b = 3)  #$ error[E1030]: field `b` is given twice
    r = P(a = 1, 2, b = 3)  #$ error[E2020]: positional arguments must come before named ones
