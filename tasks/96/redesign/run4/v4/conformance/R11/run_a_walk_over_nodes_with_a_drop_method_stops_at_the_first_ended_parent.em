#$ test: run-pass
#$ rules: R11, R10, R12, R13
#$ stdout: visit mid
#$ stdout: end mid
#$ stdout: depth 1
## R11: with a `drop` method of its own, `Node` gets no deferred-drop region (R11 places one only when
## the node's class has no `drop` method) and its parent links no proof (condition 1), so every read
## tests the header and the walk's link keeps a weak count on every target: "one move of the link's weak
## count from the node to its parent" per step (AN-62, part 6). The walk stops at the first ended parent
## all the same: `mid` is ended the moment `clear` lets it go, before its `drop` runs (verdict 56), and
## the link's weak count keeps its block for the test. Its `drop` changes nothing outside the node, so
## its first line is `fn drop():` (verdict 69). Checked under AddressSanitizer: no report.

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node) changes children:
        child.parent = weakref this
        this.children.push(child)

    fn drop():
        if this.name == "mid":
            println("end mid")

class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node) changes Node

    fn depth_total() changes on_visit, Node -> int:
        depth = 0
        for leaf in this.leaves:
            up = weakref leaf
            while up exists as n:
                if n.parent exists as p:
                    up = weakref p
                    this.on_visit(p)
                    depth += 1
                else:
                    break
        return depth

fn program() changes Node:
    root = Node(name=String("root"))
    root.add(Node(name=String("top")))
    root.children[0].add(Node(name=String("mid")))
    root.children[0].children[0].add(Node(name=String("leaf")))
    leaf = weakref root.children[0].children[0].children[0]

    fn prune(n: Node) with weakref root changes Node:
        println(f"visit {n.name}")
        if n.name == "mid":
            if root exists as ref mut r:
                r.children.clear()             # ends `top`, and `mid` and `leaf` under it

    t = Tree(root=root, leaves=[leaf], on_visit=prune)
    println(f"depth {t.depth_total()}")

fn main() changes Node:
    program()
