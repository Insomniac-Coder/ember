#$ test: run-pass
#$ rules: R3, R11, R13
#$ stdout: 7
## R3, R11, R13: the nearest program that is allowed: a class whose children are held by their parent
## and whose `parent` is a `weakref` link the program sets (AN-41's `Node`).

class Node:
    value: int
    children: Array[Node] = []
    parent: weakref Node = None

    fn add(owns child: Node) changes children:
        child.parent = weakref this
        this.children.push(child)

fn main():
    root = Node(value=7)
    root.add(Node(value=1))
    if root.children[0].parent exists as p:
        println(p.value)
