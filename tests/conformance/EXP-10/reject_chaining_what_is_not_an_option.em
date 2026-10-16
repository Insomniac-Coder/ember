#$ test: compile-fail
#$ rules: EXP-10
# `[EXP-10]` — `?.` reads through an `Option` only; on a value that is always there it is `.`.

struct P:
    x: int

fn main():
    p = P(1)
    println(p?.x)    #$ error[E2020]: `?.` needs an `Option`, not `P`
