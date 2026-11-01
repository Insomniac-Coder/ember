#$ test: compile-fail
#$ rules: OWN-9, CTL-3
# `[OWN-9]` — handing a loop's number over ends it for the rest of that turn.

fn take(owns x: int):
    println(x)

fn main():
    for i in 0..3:
        take(i)
        println(i)  #$ error[E3040]
