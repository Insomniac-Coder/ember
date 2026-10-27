#$ test: compile-pass
#$ rules: R11
## R11: the nearest allowed program to `reject_a_parent_field_that_owns_its_parent.em` and to
## `reject_a_struct_node_keeping_a_view_of_its_parent.em`: a class with a `weakref` parent link, set
## by the program before the child is put where its parent owns it (verdict 14; AN-41, AN-63).

class Node:
    name: String
    children: Array[Node] = []
    parent: weakref Node = None

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(child)

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("a")))
    if root.children[0].parent exists as p:
        println(p.name)
