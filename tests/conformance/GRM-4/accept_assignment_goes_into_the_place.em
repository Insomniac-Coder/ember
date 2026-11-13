#$ test: run-pass
#$ rules: GRM-4
#$ stdout: 1 3
## `[GRM-4]` — where `x` is in scope `x = e` assigns: the new value goes into the place `x` stands for,
## and a name from an enclosing block that is assigned in an inner block keeps the value after it.

fn main():
    x = 1
    y = 2
    if true:
        y = 3
    println(x, y)
