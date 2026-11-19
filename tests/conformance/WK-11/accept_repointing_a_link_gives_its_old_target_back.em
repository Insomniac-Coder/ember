#$ test: run-pass
#$ rules: WK-11, WK-2, WK-12
#$ profiles: debug, development, release
#$ stdout: 1
#$ 2
#$ gone
#$ 2
# `[WK-11]` — every link holds one weak count on its target: giving it another target subtracts one
# from the old target's, so the old block is freed when its object has ended (the debug profile
# reports any block a link kept), and the link names only the new target.

class Node:
    v: int

class Holder:
    keep: weakref Node = None

fn main():
    first = Node(v=1)
    h = Holder()
    h.keep = weakref first
    if h.keep exists as n:
        println(n.v)
    second = Node(v=2)
    h.keep = weakref second
    first = Node(v=3)
    if h.keep exists as n:
        println(n.v)
    h.keep = weakref first
    first = Node(v=4)
    if h.keep exists as n:
        println(n.v)
    else:
        println("gone")
    h.keep = weakref second
    if h.keep exists as n:
        println(n.v)
