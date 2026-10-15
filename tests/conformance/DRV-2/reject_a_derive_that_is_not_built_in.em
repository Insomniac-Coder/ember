#$ test: compile-fail
#$ rules: DRV-2, DRV-1
#$ error[E0104]: `Shiny` is not a built-in derive
#$ help: user-defined derives are not in this version
# `[DRV-2]` — user-defined derives are not in this version, so a derive that is not one of
# `[DRV-1]`'s built-in ones is refused for what it is, and never as something still to come.

@derive(Shiny)
struct Point:
    x: int

fn main():
    println(Point(1).x)
