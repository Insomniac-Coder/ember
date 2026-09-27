#$ test: run-pass
#$ rules: FN-5, FN-9, RC-5
#$ stdout: 5
# A default may inspect a mut handle parameter before the call; the call still
# uses the retained field handle and writes a replacement back afterward.

class Node:
    value: int

class Parent:
    child: Node

fn replace(mut node: Node, next: int = node.value + 1):
    node = Node(next)

fn main():
    parent = Parent(Node(4))
    replace(parent.child)
    println(parent.child.value)
