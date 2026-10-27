#$ test: compile-fail
#$ rules: R11, R1
## R11: a link back to the parent is a `weakref` field the program sets; the Python spelling, a strong
## field, would hand over a node that `this` does not own (AN-63, still refused).

class Node:
    name: String
    children: Array[Node] = []
    parent: Option[Node] = None

    fn add(owns child: Node):
        child.parent = Some(this)        #$ error[E3013]: `child.parent = Some(this)` would make `parent` a second owner of this node, and `this` is not yours to hand over; for a link back to the node whose `children` holds this one write `parent: weakref Node = None` and `child.parent = weakref this`
        this.children.push(child)

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("a")))
