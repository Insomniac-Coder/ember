#$ test: run-pass
#$ rules: FN-9, BCK-7
#$ profiles: debug, development
#$ stdout: 6 [7]
# `[FN-9]` — a mode on a class-handle parameter governs the handle, not the object: a borrowed
# handle parameter may write the object's fields, call its `mut self` methods and change what its
# fields hold, under Part VIII's exclusivity rules (D-590: each was `E3023`, as if writing
# through the handle re-pointed it). `[BCK-7]`: an access through a handle is no loan of the
# handle's local.

class Node:
    v: int
    items: Array[int]

    fn inc(mut self):
        self.v += 1

fn set(n: Node):
    n.v = 5

fn call(n: Node):
    n.inc()

fn push(n: Node):
    n.items.push(7)

fn main():
    n = Node(1, [])
    set(n)
    call(n)
    push(n)
    println(n.v, n.items)
