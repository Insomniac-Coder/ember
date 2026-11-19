#$ test: run-pass
#$ rules: OWN-13, WK-11, FN-1
#$ profiles: debug, development, release
#$ stdout: root
# `[WK-11]`, `[OWN-13]` — making a link is not a change of its target: any name may make one, a
# read-only parameter included (`up = weakref p`), and the link's kind is read from the path.

class Node:
    name: String

fn link_to(n: Node) -> weakref Node:
    return weakref n

fn main():
    root = Node(name=String("root"))
    up = link_to(root)
    if up exists as a:
        println(a.name)
