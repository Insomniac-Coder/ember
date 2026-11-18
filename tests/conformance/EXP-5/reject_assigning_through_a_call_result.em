#$ test: compile-fail
#$ rules: EXP-5
# `[EXP-5]` — a change, a `ref mut` name and a hand-over require a place: `f(x).y = 1` is `E2140`
# unless `f` returns `ref mut`.

struct P:
    y: int

fn make(x: int) -> P:
    return P(y=x)

fn main():
    make(1).y = 1    #$ error[E2140]: assignment target is not a place
