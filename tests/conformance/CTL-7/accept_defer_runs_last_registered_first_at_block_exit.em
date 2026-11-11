#$ test: run-pass
#$ rules: CTL-7
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3
#$ stdout: 4
# `[CTL-7]` — `defer:` registers a block to run when the enclosing block exits, last registered
# first; it uses the locals declared before it as they are when it runs.

fn main():
    x = 4
    println(1)
    defer:
        println(x)
    defer:
        println(3)
    x = 4
    println(2)
