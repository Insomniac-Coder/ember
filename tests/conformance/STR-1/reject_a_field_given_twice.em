#$ test: compile-fail
#$ rules: STR-1, TYP-25
# `[TYP-25]` — each field takes one argument: a positional argument and a
# named one for the same field, or the same name twice, is `E2021` (N171, it was `E1030`); a
# positional argument after a named one is `E2021` too (it was `E2020`).

struct P:
    a: int
    b: int

fn main():
    p = P(1, a = 2, b = 3)  #$ error[E2021]: `P(1, a = 2, b = 3)` gives `a` twice: `P(a: int, b: int)`
    q = P(a = 1, b = 2, b = 3)  #$ error[E2021]: `P(a = 1, b = 2, b = 3)` gives `b` twice: `P(a: int, b: int)`
    r = P(a = 1, 2, b = 3)  #$ error[E2021]: `P(a = 1, 2, b = 3)` gives `2` after the named `a = 1`, and positional arguments come first; write `P(a = 1, b=2, b = 3)`
#$ error[E2021]: `P(a = 1, 2, b = 3)` gives `b` twice: `P(a: int, b: int)`
