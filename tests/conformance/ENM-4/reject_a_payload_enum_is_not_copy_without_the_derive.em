#$ test: compile-fail
#$ rules: ENM-4, STR-5
#$ error[E3040]: `a`
# `[ENM-4]` — a payload enum is `Copy` only by `@derive(Copy)`: without it `=` from a whole name
# hands the value over.

enum Num:
    Int(String)
    Nothing

fn main():
    a = Num.Int(String("x"))
    b = a
    match a:
        Num.Int(s):
            println(s)
        Num.Nothing:
            pass
    match b:
        Num.Int(s):
            println(s)
        Num.Nothing:
            pass
