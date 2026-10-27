#$ test: compile-fail
#$ rules: TYP-14
#$ help: for a link back to the parent make `Node` a class with `children: Array[Node]` and `parent: weakref Node = None`, set with `child.parent = weakref this` before `this.children.push(child)`
## design rules: R3, R7
## R3, R7: a struct cannot keep a view of the struct that holds it (E2200); the link back to a parent
## is a `weakref` field of a class (AN-41's message).

struct Node:
    value: int
    children: Array[Node] = []
    parent: Option[ref Node] = None    #$ error[E2200]: `Node.parent` would be a view of the `Node` that holds this one, which no struct can keep

fn main():
    n = Node(value=0)
    println(n.value)
