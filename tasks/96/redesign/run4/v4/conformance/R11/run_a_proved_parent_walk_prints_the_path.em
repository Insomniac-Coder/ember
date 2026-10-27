#$ test: run-pass
#$ rules: R11, R13
#$ stdout: root/a/leaf
#$ stdout: leaf
#$ stdout: root/leaf
## R11: parent links are ordinary `weakref` fields the program sets (verdict 14), and "A read of one is
## compiled with no header test (a plain load and a test for no link) where the compiler proves its
## target lives": the only line that gives `parent` a target (`add`) is followed by a push into the
## parent's own list, the only line that takes a node out (`take`) is followed by `child.parent = None`,
## nodes have one owner, `Node` has no `drop` method, and nothing in `path` can end a node (AN-63).
## Section 2: "a load and a test for no link, no header test". Checked in the emitted C: `path`'s
## read of `n.parent` has no header test; under AddressSanitizer, no report.

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node) changes children:
        child.parent = weakref this            # the link back, which the program sets
        this.children.push(child)              # then the child is put where its parent owns it

    fn take(i: int) changes children -> Node:
        child = this.children.remove(i)        # out of its place, not ended
        child.parent = None                    # its link cleared before anything reads it
        return child

fn path(n: Node) -> String:
    if n.parent exists as p:
        return f"{path(p)}/{n.name}"
    return copy of n.name

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("a")))
    root.children[0].add(Node(name=String("leaf")))
    println(path(root.children[0].children[0]))     # root/a/leaf
    moved = root.children[0].take(0)
    println(path(moved))                            # leaf
    root.add(moved)
    println(path(root.children[1]))                 # root/leaf
