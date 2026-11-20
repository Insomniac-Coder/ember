# Clause BRW-21.9: "the refusal stands on the line that stores it, not on the call through the
# value, which is right for every other function"
#$ test: run-pass
#$ rules: BRW-21, DIA-26, WK-18
#$ stdout: leaf
#$ stdout: leaf
#$ stdout: 0
# Every function stored in `on_visit` uses its node before anything that may end it, so the call
# through the value and every store compile.

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

fn prune_after_reading(n: Node):
    println(n.name)
    if n.parent exists as ref mut p:
        p.children.clear()

fn main():
    root = Node("root")
    root.add(owns Node("leaf"))
    t = Tree(root)
    t.visit_first()
    t.on_visit = prune_after_reading
    t.visit_first()
    println(t.root.children.len())
