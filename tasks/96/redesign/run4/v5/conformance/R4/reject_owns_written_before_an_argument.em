#$ test: compile-fail
#$ rules: OWN-9
#$ help: x = take(a)
## design rules: R4, R3
## R4: a hand-over to an `owns` parameter is written as `=` writes one, with no word at the call;
## `owns` before a value its place already hands over is refused (rules section 3, E0109).

class Node:
    v: int

fn take(owns n: Node) -> int:
    return n.v

fn main():
    a = Node(v=1)
    x = take(owns a)    #$ error[E0109]: `take(a)` already hands `a` over; remove `owns`
    println(x)
