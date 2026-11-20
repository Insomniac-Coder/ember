# Clause WK-18.1: "When a walk calls a function that may change nodes (`this.on_visit(p)`, where the
# program stores in `on_visit: fn(Node)` a function that removes nodes from their parents' lists,
# and so ends them), no view of a node lives across the call (`[RC-5]`): the walk with views is
# `E3021`"
#$ test: compile-fail
#$ rules: WK-18, RC-5, BCK-7, BRW-1
# The spec's message, with this file's line numbers; `prune` is stored straight into the field, so
# "stored on line N" is that line under either reading of M6a's Q2.

class Node:
    name: String
    children: Array[Node] = []
    parent: weakref Node = None

    fn init(owns name: String):
        this.name = name

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(owns child)

fn look(n: Node):
    println(n.name)

fn prune(n: Node):
    if n.parent exists as ref mut p:
        p.children.remove(0)

class Tree:
    root: Node
    on_visit: fn(Node) = look

    fn init(owns root: Node):
        this.root = root

    fn walk_up(leaf: weakref Node):
        if leaf exists as n:
            cur = ref n
            while cur.parent exists as p:
                this.on_visit(p)    #$ error[E3021]: `this.on_visit(p)` (line 40) may end a `Node` (it may run `prune`, stored on line 48, which removes a node from its parent's list), and `p` reads a node and is used again on line 42; walk with a link, which may outlive its node: instead of `if leaf exists as n:`, `cur = ref n`, the `while` line and `cur = ref p`, write `up = weakref leaf`, then `while up exists as n:`, inside it `if n.parent exists as p:` with `up = weakref p` and `this.on_visit(p)`, and `else: break`
                println("visited")
                cur = ref p

fn main():
    root = Node(String("root"))
    root.add(owns Node(String("leaf")))
    t = Tree(root)
    t.on_visit = prune
    t.walk_up(weakref t.root.children[0])
