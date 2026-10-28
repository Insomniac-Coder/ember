#$ test: run-pass
#$ rules: R11, R10, R12, R13
#$ stdout: visit mid
#$ stdout: depth 1
## R11: "A walk across a call that may change nodes holds a link and tests it at every step", and "A
## walk up a tree stops at the first ended parent": the visit function, on `mid`, takes `top` out of
## `root` and so ends it with everything under it; the next `while up exists as n:` finds `mid` ended
## and the walk stops; it never visits an ended node, and nothing stops the program (verdicts 15, 28;
## AN-62 (b) and its C, `an62b_region.c`: "visited=m depth=1"). `Node` has no `drop` method, so the
## walk runs in a deferred-drop region (R11, rules section 0). Section 2: "one test of the node's header and
## one move of the link's weak count, per step" (in the region, the moves after the first link touch
## no count). Checked under AddressSanitizer: no report, no leak.

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node) changes children:
        child.parent = weakref this
        this.children.push(child)

class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node) changes Node            # may change nodes, remove them included

    fn depth_total() changes on_visit, Node -> int:
        depth = 0
        for leaf in this.leaves:
            up = weakref leaf                  # a link to the leaf, kept across the calls
            while up exists as n:              # the node still lives: call it `n`
                if n.parent exists as p:       # its parent, if it has one
                    up = weakref p             # the link now names the parent
                    this.on_visit(p)           # may end `p`, or a node above it with all under it
                    depth += 1
                else:
                    break
        return depth

fn main() changes Node:
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
