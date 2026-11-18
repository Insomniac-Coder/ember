#$ test: compile-fail
#$ rules: TYP-14
## `[TYP-14]` - a type that contains itself, or a view of its own type, with no block of its own between, is
## `E2200`; the child held in its own block, a list or a class is fine.

struct Tree:
    left: Option[Tree]               #$ error[E2200]: `Tree` contains itself through `left`, so a `Tree` would have no end; hold the child in its own block: `left: Option[owns Tree]`

struct Node:
    parent: ref Node                 #$ error[E2200]: `Node.parent` would be a view of the `Node` that holds this one, which no struct can keep; for a link back to the parent make `Node` a class with `children: Array[Node]` and `parent: weakref Node = None`, set with `child.parent = weakref this` before `this.children.push(owns child)`

struct Fine:
    left: Option[owns Fine]
    more: Array[Fine]

fn main():
    println(1)
