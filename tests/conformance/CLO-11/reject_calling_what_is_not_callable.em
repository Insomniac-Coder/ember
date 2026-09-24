#$ test: compile-fail
#$ rules: CLO-11, CLO-3
# `[CLO-11]` — only a field of callable type is called; `[CLO-3]` — only a
# value of callable type is. A local that is not callable is named as such
# (D-234), rather than reported as missing.

struct H:
    n: int

fn main():
    x = 3
    y = x(4)                #$ error[E2020]: `x` has type `i64`, which cannot be called
    z = (x)(4)              #$ error[E2020]: a value of type `i64` cannot be called
    h = H(n=1)
    w = h.n(2)              #$ error[E1010]: `H` has no method named `n`
