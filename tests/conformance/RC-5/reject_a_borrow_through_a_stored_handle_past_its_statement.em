#$ test: compile-fail
#$ rules: RC-5, EXP-4
# `[RC-5]` (ODR-065, SP-010) — a borrow through a handle stored in an object
# holds its own copy of the handle only to the end of the statement, since
# another handle could replace the stored one and free the object: `q` does,
# and `r` would read freed memory. Bind the handle to a local to keep it
# longer. The same holds when the handle is inside a `Copy` struct field.

class Node:
    v: int

class Parent:
    child: Node

@derive(Copy)
struct Pair:
    a: Node
    b: int

class Holder:
    pair: Pair

fn main():
    p = Parent(Node(1))
    q = p
    r = ref p.child.v #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    q.child = Node(2)
    println(r)
    h = Holder(Pair(Node(5), 0))
    g = h
    s = ref h.pair.a.v #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    g.pair = Pair(Node(6), 1)
    println(s)
