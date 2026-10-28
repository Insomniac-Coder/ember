#$ test: compile-fail
#$ rules: R11, R8, R10, R12
## R11: "When the walk calls a function that may change nodes (`on_visit: fn(Node) changes Node`) ...
## no view of a node lives across the call (R8 (g)): the walk with views is refused (E3021), and it is
## written with a `weakref` link" (verdict 28; AN-62's walk (a) given walk (b)'s visit type; the
## message is the E3021 walk row of `V4/rules.md` section 3, in this file's lines; verdict 73 keeps E3021 for
## a walk, R10 closing paragraph).

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
    on_visit: fn(Node) changes Node

    fn depth_total() changes on_visit, Node -> int:
        depth = 0
        for leaf in this.leaves:
            if leaf exists as n:
                cur = ref n
                while cur.parent exists as p:
                    this.on_visit(p)           #$ error[E3021]: `this.on_visit(p)` (line 29) may change any `Node`, and so end one (`on_visit` is a `fn(Node) changes Node`), and `p` reads a node and is used again on line 31; walk with a link, which may outlive its node: instead of `if leaf exists as n:`, `cur = ref n`, the `while` line and `cur = ref p`, write `up = weakref leaf`, then `while up exists as n:`, inside it `if n.parent exists as p:` with `up = weakref p` and `this.on_visit(p)`, and `else: break`
                    depth += 1
                    cur = ref p
        return depth

fn main() changes Node:
    root = Node(name=String("root"))
    root.add(Node(name=String("leaf")))
    leaf = weakref root.children[0]
    t = Tree(root=root, leaves=[leaf], on_visit=fn(n: Node): pass)
    println(t.depth_total())
