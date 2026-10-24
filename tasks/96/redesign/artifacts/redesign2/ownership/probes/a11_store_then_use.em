# A11: build a node, store it, keep using its name. Python prints 5.
struct Node:
    value: int
    children: Array[Node] = []

fn main():
    root = Node(value=0)
    child = Node(value=1)
    root.children.push(child)
    child.value = 5
    println(root.children[0].value)
