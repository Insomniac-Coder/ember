# Clause WK-18.2: "**A walk up a tree stops at the first ended parent**: if the call ends `p`, or a
# node above it with everything under it, the link's target is ended and its block is kept by the
# link's weak count, so the next `while up exists as n:` finds it ended and the walk stops"; "an
# object whose last owner ends during the walk is ended at once, as anywhere: the `drop`s of what it
# owns run then, in their usual order, so the program prints what it would with no region"
#$ test: run-pass
#$ rules: WK-18, WK-3, CLS-6
#$ stdout: mid
#$ stdout: tag leaf ends
#$ stdout: tag mid ends
#$ stdout: walked
#$ stdout: 0
# The spec's walk. Visiting `mid` clears the root's list, which ends `mid` and the `leaf` under it;
# the walk then finds `mid` ended and stops. `Node` has no `drop`, so the walk may be a
# deferred-drop region, but each `Tag`'s `drop` runs at the clear, before "walked" (the lead's ruling
# of area 3's Q3, now in the clause).

class Tag:
    name: String

    fn init(owns name: String):
        this.name = name

    fn drop():
        println(f"tag {this.name} ends")

class Node:
    tag: Tag
    name: String
    children: Array[Node] = []
    parent: weakref Node = None

    fn init(owns name: String):
        this.tag = Tag(copy of name)
        this.name = name

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(owns child)

fn visit(n: Node):
    println(n.name)
    if n.name == "mid":
        if n.parent exists as ref mut p:
            p.children.clear()

class Tree:
    root: Node
    on_visit: fn(Node) = visit

    fn init(owns root: Node):
        this.root = root

    fn walk_up(leaf: weakref Node):
        up = weakref leaf
        while up exists as n:
            if n.parent exists as p:
                up = weakref p
                this.on_visit(p)
            else:
                break
        println("walked")

fn main():
    root = Node(String("root"))
    mid = Node(String("mid"))
    mid.add(owns Node(String("leaf")))
    root.add(owns mid)
    t = Tree(root)
    t.walk_up(weakref t.root.children[0].children[0])
    println(t.root.children.len())
