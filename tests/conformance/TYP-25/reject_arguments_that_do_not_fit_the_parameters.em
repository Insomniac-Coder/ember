#$ test: compile-fail
#$ rules: TYP-25, DIA-22
# `[TYP-25]` (N171) - a call whose arguments do not fit the callee's parameters is `E2021`, and the
# message gives the callee's first line. For a positional argument after a named one, the line to
# write names it with the first parameter no argument before it gives.

fn area(w: int, h: int, scale: int = 1) -> int:
    return w * h * scale

fn main():
    println(area(2))    #$ error[E2021]: `area(2)` gives nothing for `h`, which has no default: `fn area(w: int, h: int, scale: int = 1) -> int`
    println(area(w=2, 3))    #$ error[E2021]: `area(w=2, 3)` gives `3` after the named `w=2`, and positional arguments come first; write `area(w=2, h=3)`
    println(area(1, 2, 3, 4))    #$ error[E2021]: `area(1, 2, 3, 4)` gives 4 arguments, and `area` takes 3: `fn area(w: int, h: int, scale: int = 1) -> int`
    println(area(1, 2, depth=3))    #$ error[E2021]: `area(1, 2, depth=3)` names `depth`, and `area` has no parameter of that name
    println(area(1, w=2, h=3))    #$ error[E2021]: `area(1, w=2, h=3)` gives `w` twice
