#$ test: compile-fail
#$ rules: R4, R1
#$ help: v = a.v
## R4: `owns n: Node` takes the caller's object over: the caller's name ends at the call, and a later
## use of it is refused (AN-43).

class Node:
    v: int

fn take(owns n: Node) -> int:
    return n.v

fn main():
    a = Node(v=1)
    x = take(a)
    println(a.v, x)    #$ error[E3040]: `a` was handed over to `take` on line 15 and `a.v` reads it on line 16; read it before the call (`v = a.v` above line 15), or give `take` a copy of its own: add `@derive(Clone)` above `class Node` and write `take(copy of a)`
