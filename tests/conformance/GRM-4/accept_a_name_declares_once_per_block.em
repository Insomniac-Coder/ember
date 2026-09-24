#$ test: run-pass
#$ rules: GRM-4
#$ stdout:
#$ 5
#$ 2 3
# `[GRM-4]` — `x = e` declares when no `x` is in scope and assigns when one
# is; `x: T = e` always declares, and may shadow a name from an enclosing
# block or a parameter.

fn f(x: int) -> int:
    x: int = 2
    return x

fn main():
    y: int = 1
    if true:
        y: int = 5
        println(y)
    y = 3
    println(f(1), y)
