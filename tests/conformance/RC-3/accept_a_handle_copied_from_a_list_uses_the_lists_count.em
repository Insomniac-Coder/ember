#$ test: run-pass
#$ rules: RC-3, RC-1, RC-2a, EXC-15
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ stdout: 45
#$ end
#$ drop 6
#$ drop 7
# `[RC-3]` — `t` is a copy of an element of `nodes`, which nothing changes
# while `t` lives, so the list's own count keeps the object alive: the copy
# is not retained and `t`'s end releases nothing. Lending `t` to a `mut self`
# method or a borrowed parameter cannot re-point it. Fresh constructor
# temporaries transfer into the list, so insertion also needs no retain.

class Node:
    v: int

    fn bump(mut self):
        self.v += 1

    fn drop(mut self):
        println("drop", self.v)

fn value_of(n: Node) -> int:
    return n.v

fn main():
    nodes: Array[Node] = []
    nodes.push(Node(1))
    nodes.push(Node(2))
    total = 0
    for i in 0..10:
        t = nodes[i % 2]
        t.bump()
        total += value_of(t)
    println(total)
    println("end")
