# A11: a field of a moved-from struct is assigned, then the struct goes out of scope.
# What does the emitted C drop at the end of main?
struct Node:
    value: int
    children: Array[Node] = []

fn main():
    root = Node(value=0)
    child = Node(value=1)
    root.children.push(child)
    child.value = 5
    println(root.children[0].value)
    println(child.value)
