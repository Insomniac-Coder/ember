#$ test: run-pass
#$ rules: GRM-42, CLO-2
#$ stdout: 5
#$ stdout: 5
## `[GRM-42]`, `[CLO-2]` — a name a lambda uses with no capture word is the real variable: the lambda
## reads it as it is when called, and may change it.

fn main():
    total = 0
    add = fn(x: int): total += x
    add(2)
    add(3)
    println(total)
    read = fn() => total
    println(read())
