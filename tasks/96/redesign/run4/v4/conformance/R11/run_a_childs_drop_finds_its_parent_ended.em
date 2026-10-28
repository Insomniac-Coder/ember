#$ test: run-pass
#$ rules: R11, R13, R5
#$ stdout: end a under root
#$ stdout: root ends
#$ stdout: end b, its parent already ended
## R11 (verdict 56): a `weakref` link opened "during the `drop` of anything it owns (a child's `drop`
## opening `this.parent` while the parent is being ended), finds it ended", and "Ending the link itself
## (`this.parent = None`) stays allowed: it only subtracts the link's weak count in the target's
## header". `a` ends while `root` lives and finds its parent; `b` ends with `root` and finds it ended.
## `Node` has a `drop`, so its parent reads are not proved and each tests the header (R11, condition 1).
## Checked under AddressSanitizer: no report.

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node) changes children:
        child.parent = weakref this
        this.children.push(child)

    fn drop():
        if this.parent exists as p:
            println(f"end {this.name} under {p.name}")
        elif this.name != "root":
            println(f"end {this.name}, its parent already ended")
        this.parent = None                  # ends the link only: allowed

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("a")))
    root.add(Node(name=String("b")))
    root.children.remove(0)                 # ends `a` while `root` lives
    println("root ends")
