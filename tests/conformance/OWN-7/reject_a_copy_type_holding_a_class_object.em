#$ test: compile-fail
#$ rules: OWN-7, TYP-45
# `[TYP-45]` — a class object has one owner and is not `Copy`, so a type holding one is not `Copy` either.

class Node:
    v: int

@derive(Copy)
struct Pair:
    a: Node    #$ error[E2080]: field `a` has type `Node`, which is not Copy
    b: int

fn main():
    println(Pair(Node(1), 2).b)
