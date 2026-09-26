#$ test: run-pass
#$ rules: RC-5, FN-9, EXC-17
#$ stdout: poke 1
#$ drop 1
#$ bump 2
#$ drop 2
#$ kept 3
#$ drop 3
#$ end
#$ drop 7
#$ drop 8
#$ drop 9
# `[RC-5]` (ODR-065) — a handle stored in an object (`p.child`) can be
# overwritten through another handle while a call uses the object. The call
# is made on a retained copy, so the object lives until the call returns:
# `poke` and `bump` replace `p.child` and still read their own object, which
# is dropped only after. A handle bound to a local keeps its object as long
# as the local (`c`). Before ODR-065 each object was dropped mid-use and the
# method read freed memory.

class Node:
    v: int

    fn poke(self, owned p: Parent):
        p.child = Node(9)
        println("poke", self.v)

    fn bump(mut self, owned p: Parent):
        r = ref self.v
        p.child = Node(8)
        println("bump", r)

    fn drop(mut self):
        println("drop", self.v)

class Parent:
    child: Node

fn main():
    p = Parent(Node(1))
    p.child.poke(p)
    q = Parent(Node(2))
    q.child.bump(q)
    s = Parent(Node(3))
    c = s.child
    r = ref c.v
    s.child = Node(7)
    println("kept", r)
    mem.drop(c)
    println("end")
