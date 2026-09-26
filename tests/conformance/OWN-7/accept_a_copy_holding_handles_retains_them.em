#$ test: run-pass
#$ rules: OWN-7, STR-3, RC-1
#$ stdout: 1 1 2
#$ 5 5
#$ drop 5
#$ drop 1
# `[OWN-7]`, `[STR-3]` (ODR-051, SP-010) — a struct whose fields are all
# `Copy` may derive `Copy` though a field is a class handle; a copy of it,
# or of a tuple of handles, retains each handle, so every object is dropped
# once, when its last copy goes.

class Node:
    v: int

    fn drop(mut self):
        println("drop", self.v)

@derive(Copy)
struct Pair:
    a: Node
    b: int

fn main():
    p = Pair(Node(1), 2)
    q = p
    println(p.a.v, q.a.v, q.b)
    t = (Node(5), 1)
    u = t
    println(t.0.v, u.0.v)
