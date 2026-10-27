#$ test: compile-fail
#$ rules: BRW-1
#$ help: up = weakref leaf
## design rules: R11, R8, R10, R12
## R11: "When the walk calls a function that may change nodes (`this.on_visit(p)`, where the program
## stores in `on_visit: fn(Node)` a function that removes nodes from their parents' lists, and so ends
## them), no view of a node lives across the call (R8 (g)): the walk with views is refused (E3021)"
## (verdicts 28, 86; AN-62's walk (a) given walk (b)'s `prune`). The message is the E3021 walk row of
## rules section 3, in this file's lines; verdict 73 keeps E3021 for a walk (R10's closing paragraph).

class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(child)

class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node)

    fn depth_total() -> int:
        depth = 0
        for leaf in this.leaves:
            if leaf exists as n:
                cur = ref n
                while cur.parent exists as p:
                    this.on_visit(p)           #$ error[E3021]: `this.on_visit(p)` (line 30) may end a `Node` (it may run `prune`, stored on line 46, which clears the children of `root`, line 44), and `p` reads a node and is used again on line 32; walk with a link, which may outlive its node: instead of `if leaf exists as n:`, `cur = ref n`, the `while` line and `cur = ref p`, write `up = weakref leaf`, then `while up exists as n:`, inside it `if n.parent exists as p:` with `up = weakref p` and `this.on_visit(p)`, and `else: break`
                    depth += 1
                    cur = ref p
        return depth

fn main():
    root = Node(name=String("root"))
    root.add(Node(name=String("top")))
    root.children[0].add(Node(name=String("leaf")))
    leaf = weakref root.children[0].children[0]

    fn prune(n: Node) weakref root:
        println(f"visit {n.name}")
        if root exists as ref mut r:
            r.children.clear()

    t = Tree(root=root, leaves=[leaf], on_visit=prune)
    println(t.depth_total())
