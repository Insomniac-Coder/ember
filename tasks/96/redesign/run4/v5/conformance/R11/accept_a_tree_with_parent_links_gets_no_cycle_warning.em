#$ test: compile-pass
#$ rules: R11
## R11, AN-64: a list whose elements have one owner cannot form a cycle, so the tree with `weakref`
## parent links gets no warning (today's compiler gives L3001 on `children` and advises making the
## owning edge weak, `tests/conformance/WK-5`). The runner expects no warning.

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(child)

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("a")))
