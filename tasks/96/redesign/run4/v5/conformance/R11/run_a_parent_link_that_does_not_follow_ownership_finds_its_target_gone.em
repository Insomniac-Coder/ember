#$ test: run-pass
#$ rules: WK-11, GRM-44
#$ stdout: other/leaf
#$ stdout: leaf
## design rules: R11, R13
## R11: "where condition 1 fails, every read tests": a line here gives a node's `parent` a target that
## does not own it (`root.children[0].parent = weakref other`), so the proof fails for the whole
## package, the program is accepted, and every read of `parent` tests the target's header, which finds
## `other` ended (AN-63: "the program stays safe and gets slower"). Checked in the emitted C: `path`'s
## read tests the header; under AddressSanitizer, no report.

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(child)

fn path(n: Node) -> String:
    if n.parent exists as p:
        return f"{path(p)}/{n.name}"
    return copy of n.name

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("leaf")))
    other = Node(name=String("other"))
    root.children[0].parent = weakref other         # a link its target does not own
    println(path(root.children[0]))                 # other/leaf
    other = Node(name=String("next"))               # the node the link names ends
    println(path(root.children[0]))                 # leaf
