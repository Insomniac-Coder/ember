struct Node:
    children: Array[Node]

fn main():
    n = Node(children=[])
    println(n.children.len())
