#$ test: compile-fail
#$ rules: GRM-12, TYP-5
## `[GRM-12]` — a constant in a pattern matches as the literal it stands for,
## of the matched type: widening is for coercion sites, and a pattern is not
## one.

const SMALL: i32 = 3

fn main():
    x: i64 = 3
    match x:
        SMALL:    #$ error[E2020]: the constant `SMALL` cannot match a value of type `i64`
            println("small")
        _:
            println("other")
