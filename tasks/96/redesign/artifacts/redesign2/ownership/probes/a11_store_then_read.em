# A11: the same, but the name is read after the store.
struct Node:
    value: int
    children: Array[Node] = []

fn main():
    root = Node(value=0)
    child = Node(value=1)
    root.children.push(child)
    child.value = 5
    println(child.value)
