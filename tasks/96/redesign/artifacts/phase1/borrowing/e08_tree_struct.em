# A8: a tree of plain structs with a parent reference.
struct Node:
    name: String
    parent: Option[ref Node]
    children: Array[Node]

fn build() -> Node:
    root = Node(name="root", parent=None, children=[])
    leaf = Node(name="leaf", parent=Some(ref root), children=[])
    root.children.push(leaf)
    return root

fn main():
    println(build().children.len())
