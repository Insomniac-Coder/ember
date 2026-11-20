# Clause WK-1.1: "Ember has no cycle collector: a loop of owners (...) would keep its objects alive
# for ever. So a loop of owners is allowed only where the compiler proves it is opened again (...);
# a link back that does not keep its target alive is a `weakref` (`[WK-11]`)."
#$ test: run-pass
#$ rules: WK-1, WK-11, CLS-6
#$ stdout: b a
#$ stdout: a ends
#$ stdout: b ends
#$ stdout: done
# `a` owns `b`, and `b` links back with `weakref`: both `drop`s run when the function ends (its
# locals end in reverse order; a node's `drop` runs before its fields end, `[CLS-6]`).

class Node:
    name: String
    next: Option[shares mut Node] = None
    prev: weakref Node = None

    fn init(owns name: String):
        this.name = name

    fn drop():
        println(f"{this.name} ends")

fn build():
    a = shares mut Node(String("a"))
    b = shares mut Node(String("b"))
    a.next = Some(shares mut b)
    b.prev = weakref a
    if b.prev exists as p:
        println(f"{b.name} {p.name}")

fn main():
    build()
    println("done")
