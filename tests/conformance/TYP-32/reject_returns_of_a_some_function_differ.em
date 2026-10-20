#$ test: compile-fail
#$ rules: TYP-32
#$ error[E2261]: this returns `str`, but an earlier `return` gave `i64`
# `[TYP-32]` — every `return` in a `some` function's body produces the same concrete type: one
# giving an `int` and another a `str` is `E2261`, though both implement `Display`.

fn pick(big: bool) -> some Display:
    if big:
        return 100
    return "small"

fn main():
    println(pick(true))
