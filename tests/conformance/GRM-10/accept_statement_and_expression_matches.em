#$ test: run-pass
#$ rules: GRM-10
#$ stdout: ten
#$ stdout: 10
## `[GRM-10]` — arms written `pattern: block` make a statement, and arms
## written `pattern => e` an expression.

fn main():
    x = 1
    match x:
        1:
            println("ten")
        _:
            println("other")
    y = match x:
        1 => 10
        _ => 0
    println(y)
