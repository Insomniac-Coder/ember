#$ test: run-pass
#$ rules: R9, R3
#$ stdout: 5
#$ stdout: 6
#$ warning[L3001]: `Node.children` holds owners of nodes, so a node put among its own children (`child.children.push(shares mut child)`) would own itself and never be freed; push each node under one parent only, as `root.children.push(shares mut child)` does; a link back up to a parent is one that does not keep it alive: `parent: weakref Node = None`
## R9: a child with two owners, the tree and the function that made it, is changed through either, and
## each change is seen through the other; `child` keeps what it could do after it is shared
## (AN-36's second form, verdict 23). The list of `shares mut` owners of its own type gets R11's
## cycle warning, AN-36's text.

struct Node:
    value: int
    children: Array[shares mut Node] = []

fn main():
    root = Node(value=0)
    child = Node(value=1)
    root.children.push(shares mut child)
    child.value = 5
    println(root.children[0].value)     # 5
    root.children[0].value = 6
    println(child.value)                # 6
