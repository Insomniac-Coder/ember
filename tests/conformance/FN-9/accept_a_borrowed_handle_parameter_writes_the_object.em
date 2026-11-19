#$ test: run-pass
#$ rules: FN-9, BCK-7
#$ profiles: debug, development
#$ stdout: 6 [7]
# `[FN-9]` — a class object has one owner, so `ref mut n: Node` may change the object: write its
# fields, call its `mut self` methods and change what its fields hold (a plain `n: Node` only reads it,
# `reject_a_reading_class_parameter_changes_nothing.em`). `[BCK-7]`: an access through a handle is no
# loan of the handle's local.

class Node:
    v: int
    items: Array[int]

    fn inc():
        this.v += 1

fn set(ref mut n: Node):
    n.v = 5

fn call(ref mut n: Node):
    n.inc()

fn push(ref mut n: Node):
    n.items.push(owns 7)

fn main():
    n = Node(1, [])
    set(ref mut n)
    call(ref mut n)
    push(ref mut n)
    println(n.v, n.items)
