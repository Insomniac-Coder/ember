#$ test: run-pass
#$ rules: WK-11, CLO-1, GRM-44
#$ stdout: visit mid
#$ stdout: visit root
#$ stdout: depth 2
## design rules: R11, R12, R13
## R11: the walk with a link reads each node's parent afresh at every step, so when the visit function
## moves `p` under another parent (taking it out and giving it a new link, AN-63's `take` and `add`)
## "the walk goes on up the new parents" (AN-62 (b)): `top`, `mid`'s old parent, is never visited.
## `this.on_visit(p)` compiles: `regraft` ends no node, and moves `mid` only after its last use of it
## (verdict 53).

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(child)

    fn take(i: int) -> Node:
        child = this.children.remove(i)
        child.parent = None
        return child

class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node)

    fn depth_total() -> int:
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

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("top")))
    root.children[0].add(Node(name=String("mid")))
    root.children[0].children[0].add(Node(name=String("leaf")))
    leaf = weakref root.children[0].children[0].children[0]

    fn regraft(n: Node) weakref root:
        println(f"visit {n.name}")
        if n.name == "mid":
            if root exists as ref mut r:
                moved = r.children[0].take(0)  # `mid` out of `top`, its link cleared
                r.add(moved)                   # `mid` under `root`, its link set to `root`

    t = Tree(root=root, leaves=[leaf], on_visit=regraft)
    println(f"depth {t.depth_total()}")
