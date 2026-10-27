#$ test: compile-pass
#$ rules: R11, R5
## R11: "That count is a number in the target's header that no name reads, so making or ending a link
## is not a change of the target: any name may make one (`up = weakref p` from a read-only `p`;
## `child.parent = weakref this` in a method that otherwise changes only `this.children`)" (AN-41,
## AN-63).

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node):
        child.parent = weakref this      # a link to `this`: not a change of `this`
        this.children.push(child)

fn link_to(n: Node) -> weakref Node:
    return weakref n                     # from a read-only parameter: not a change of `n`

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("a")))
    up = link_to(root.children[0])
    if up exists as a:
        println(a.name)
