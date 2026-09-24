#$ test: compile-fail
#$ rules: STR-2, DIA-14
# `[STR-2]` — a default is evaluated where the struct is declared, so the
# constructing function's `y` is not in scope; `[DIA-14]` — one mistake in a
# default is one error, however many constructions evaluate it.

struct D:
    x: int = y + 1          #$ error[E1010]: cannot find `y` in this scope

fn main():
    y = 5
    d = D()
    e = D()
    println(d.x, e.x, y)
