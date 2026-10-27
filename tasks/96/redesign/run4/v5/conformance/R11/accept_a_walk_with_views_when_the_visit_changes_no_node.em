#$ test: compile-pass
#$ rules: WK-11, RC-5, CLO-1
## design rules: R11, R10, R12
## R11: "A walk whose visit function changes no node (every function the program stores in `on_visit`
## only reads its node) keeps its views and pays nothing per step" (verdict 28; AN-62 (a)): the only
## function stored there, `fn(n: Node): pass`, changes nothing and ends no `Node` (R12, verdict 86), so
## no line can end a `Node` (R10 point 1), and the parent link follows ownership, so each step is a
## plain load of `cur.parent` and a test for no link; the one header test is `if leaf exists as n:`,
## once per leaf. Checked in the emitted C: no header test, no count and no region counter inside the
## `while` loop.

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(child)

class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node)                         # what it does is worked out from what is stored in it

    fn depth_total() -> int:
        depth = 0
        for leaf in this.leaves:
            if leaf exists as n:
                cur = ref n
                while cur.parent exists as p:
                    this.on_visit(p)
                    depth += 1
                    cur = ref p
        return depth

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("leaf")))
    leaf = weakref root.children[0]
    t = Tree(root=root, leaves=[leaf], on_visit=fn(n: Node): pass)
    println(t.depth_total())
