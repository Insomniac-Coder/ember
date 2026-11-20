# Clause DIA-26.1: "One mistake gives one message, at the program's line, on the name that caused
# it. A fault inside a function stored in a function value (it ends what its parameter views and
# then uses the parameter) is reported on the line that stores it, not on a call through the value."
#$ test: compile-fail
#$ rules: DIA-26, BRW-21
# `prune` is stored once and called through the value from two places: one message, at the store.
# No text in the spec for this program: code only.

class Node:
    name: String
    children: Array[Node] = []
    parent: weakref Node = None

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(owns child)

class Tree:
    root: Node
    on_visit: fn(Node) = look

    fn init(owns root: Node):
        this.root = root

    fn visit_first():
        if this.root.children.len() > 0:
            this.on_visit(this.root.children[0])

    fn visit_last():
        n = this.root.children.len()
        if n > 0:
            this.on_visit(this.root.children[n - 1])

fn look(n: Node):
    println(n.name)

fn prune(n: Node):
    if n.parent exists as ref mut p:
        p.children.clear()
    println(n.name)

fn main():
    root = Node("root")
    root.add(owns Node("leaf"))
    t = Tree(root)
    t.on_visit = prune    #$ error[E3031]
    t.visit_first()
    t.visit_last()
