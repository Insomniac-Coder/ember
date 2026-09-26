#$ test: run-pass
#$ rules: FN-9, RC-5
#$ stdout: drop 1
#$ child 42
#$ drop 3
#$ local 42
#$ r 42
#$ drop 9
#$ after 42
#$ drop 42
#$ drop 42
# `[FN-9]` — a `mut` handle parameter lets the callee re-point the caller's
# handle, a local's or a field's (D-349: re-pointing panicked). `[RC-5]`
# (ODR-065) — a handle stored in an object is passed as a retained copy and
# stored back after the call, so the callee may borrow through it for as long
# as it likes: `swap_in` replaces `p.child` through `p` while `r` points into
# the object it was given, which lives until the call returns; the store back
# then replaces the `Node(9)` it wrote.

class Node:
    v: int

    fn drop(mut self):
        println("drop", self.v)

class Parent:
    child: Node

fn repoint(mut n: Node):
    n = Node(42)

fn swap_in(mut n: Node, owned p: Parent):
    r = ref n.v
    p.child = Node(9)
    println("r", r)

fn main():
    p = Parent(Node(1))
    repoint(p.child)
    println("child", p.child.v)
    local = Node(3)
    repoint(local)
    println("local", local.v)
    swap_in(p.child, p)
    println("after", p.child.v)
