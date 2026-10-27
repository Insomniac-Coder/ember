#$ test: compile-fail
#$ rules: BRW-21, FFI-52
#$ help: move line 51 above line 48
## design rules: R11, R4, R12
## R11, R4 (verdict 53, its corrected record): the walk's call `this.on_visit(p)` compiles when no
## function the program stores in `on_visit` uses its node after a line that may end it. `prune` clears
## `root`'s children, which may end the node it is given (`mid`, under `top`), and then reads that node,
## so it is refused where it is stored, not at the call through the value (E3031; rules section 3's
## E3031 message for the storing line, in this program's lines). The allowed neighbours are the two
## walk tests, `run_a_walk_whose_visit_may_end_nodes_stops_at_the_first_ended_parent.em` and
## `run_a_walk_over_nodes_with_a_drop_method_stops_at_the_first_ended_parent.em`, whose `prune` prints
## before it clears.

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

    fn prune(n: Node) weakref root:
        if n.name == "mid":
            if root exists as ref mut r:
                r.children.clear()             # ends `top`, and `mid` (`n`) and `leaf` under it
        println(f"visit {n.name}")             # reads `n`, which line 50 may have ended

    t = Tree(root=root, leaves=[leaf], on_visit=prune)    #$ error[E3031]: `on_visit` is called with a node of the tree (line 34), and `prune` may end that node (`r.children.clear()`, line 50) and then reads it (line 51); read it before: move line 51 above line 48
    println(f"depth {t.depth_total()}")
