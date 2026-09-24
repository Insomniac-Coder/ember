#$ test: compile-fail
#$ rules: TYP-30
# `[TYP-30]` — a constant negative exponent on an integer is `E2151`; an
# integer base with a float exponent, and a base that is not a number, are
# `E2020`.

fn main():
    a = 2 ** -1             #$ error[E2151]: an integer `**` with a negative exponent
    x = 3
    b = x ** 0.5            #$ error[E2020]: an integer `**` takes an integer exponent, not `f64`
    c = "s" ** 2            #$ error[E2020]: `**` takes a number, not `str`
    d = x ** (-2)           #$ error[E2151]: an integer `**` with a negative exponent
