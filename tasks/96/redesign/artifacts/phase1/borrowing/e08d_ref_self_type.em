struct Node:
    parent: Option[ref Node]

fn main():
    n = Node(parent=None)
