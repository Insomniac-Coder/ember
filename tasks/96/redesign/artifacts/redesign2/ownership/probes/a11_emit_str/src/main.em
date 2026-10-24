# A11: a String field of a moved-from struct is assigned. Does the assignment drop the old String,
# which now belongs to root.children[0]? (Read in the emitted C; nothing is run.)
struct Node:
    name: String
    children: Array[Node] = []

fn main():
    root = Node(name="root")
    child = Node(name="child")
    root.children.push(child)
    child.name = "renamed"
    println(root.children[0].name)
    println(child.name)
