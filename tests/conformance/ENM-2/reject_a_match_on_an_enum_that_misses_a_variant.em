#$ test: compile-fail
#$ rules: ENM-2, CTL-5
#$ error[E2090]: `Color.Green`, `Color.Blue` not covered
# `[ENM-2]` — a `match` on an enum MUST be exhaustive; `E2090` lists the missing variants.

enum Color:
    Red
    Green
    Blue

fn partial(c: Color) -> int:
    match c:
        Color.Red:
            return 1
    return 0

fn main():
    println(partial(Color.Red))
