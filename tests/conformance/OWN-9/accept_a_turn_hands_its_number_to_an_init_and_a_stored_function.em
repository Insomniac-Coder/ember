#$ test: run-pass
#$ rules: OWN-9, CTL-3
#$ stdout: 0
#$ 1
#$ 0
#$ 1
# `[OWN-9]` — a counted loop's turn gives its number to a class's `init` as `=` gives it, copied
# (a constructor's argument takes no word, verdict 159), and hands it over through a name holding a
# function, as to any `owns` parameter.

class Node:
    v: int

    fn init(owns x: int):
        this.v = x

fn take(owns x: int):
    println(x)

fn main():
    for i in 0..2:
        a = Node(i)
        println(a.v)
    f = take
    for i in 0..2:
        f(i)
