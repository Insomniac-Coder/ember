# Clause BRW-21.9: "Where the fault is in one function stored in a function value (it ends what its
# parameter views and then uses the parameter), the refusal stands on the line that stores it, not
# on the call through the value, which is right for every other function"
#$ test: compile-fail
#$ rules: BRW-21, DIA-26, WK-18, CLO-17
# `prune` clears the list its node is in, which ends the node, and then reads it. The call through
# `on_visit` (line 28) is right for `look` and is not refused; storing `prune` is. The spec's
# text describes its own tree program (its lines 21, 41, 42), not this one: code only.

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
    t.visit_first()
    t.on_visit = prune    #$ error[E3031]
    t.visit_first()
