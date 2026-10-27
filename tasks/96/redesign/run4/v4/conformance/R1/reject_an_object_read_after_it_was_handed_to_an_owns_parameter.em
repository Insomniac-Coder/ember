#$ test: compile-fail
#$ rules: R1, R4
#$ help: read it before the call: write `v = a.v` above line 16 and `println(v)` in place of line 17; or give `take` a copy of its own: add `@derive(Clone)` above `class Node` and write `take(copy of a)`
## R1, R4: a hand-over to an `owns` parameter is written as `=` writes one, with no word, and ends the
## caller's name for every type, a class object included (AN-43: today the class case compiles and the
## caller keeps a second owner).

class Node:
    v: int

fn take(owns n: Node) -> int:
    return n.v

fn main():
    a = Node(v=1)
    x = take(a)
    println(a.v)               #$ error[E3040]: `a` was handed over to `take` on line 16 and `a.v` reads it on line 17
    println(x)
