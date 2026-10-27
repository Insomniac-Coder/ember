#$ test: compile-fail
#$ rules: TYP-14
## design rules: R11, R7
## R11: a link back to the parent needs an object that lives on its own: a class with a `weakref`
## field; a struct cannot keep a view of the struct that holds it (AN-41, still refused).

struct Node:
    value: int
    children: Array[Node] = []
    parent: Option[ref Node] = None      #$ error[E2200]: `Node.parent` would be a view of the `Node` that holds this one, which no struct can keep; for a link back to the parent make `Node` a class with `children: Array[Node]` and `parent: weakref Node = None`, set with `child.parent = weakref this` before `this.children.push(child)`

fn main():
    pass
