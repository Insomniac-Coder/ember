#$ test: run-pass
#$ rules: R11, R12, R13
#$ stdout: visit mid
#$ stdout: visit root
#$ stdout: depth 2
## R11: the walk with a link reads each node's parent afresh at every step, so when the visit function
## moves `p` under another parent (taking it out and giving it a new link, AN-63's `take` and `add`)
## "the walk goes on up the new parents" (AN-62 (b)): `top`, `mid`'s old parent, is never visited.

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node) changes children:
        child.parent = weakref this
        this.children.push(child)

    fn take(i: int) changes children -> Node:
        child = this.children.remove(i)
        child.parent = None
        return child

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

fn main() changes Node:
    root = Node(name=String("root"))
    root.add(Node(name=String("top")))
    root.children[0].add(Node(name=String("mid")))
    root.children[0].children[0].add(Node(name=String("leaf")))
    leaf = weakref root.children[0].children[0].children[0]

    fn regraft(n: Node) with weakref root changes Node:
        println(f"visit {n.name}")
        if n.name == "mid":
            if root exists as ref mut r:
                moved = r.children[0].take(0)  # `mid` out of `top`, its link cleared
                r.add(moved)                   # `mid` under `root`, its link set to `root`

    t = Tree(root=root, leaves=[leaf], on_visit=regraft)
    println(f"depth {t.depth_total()}")
