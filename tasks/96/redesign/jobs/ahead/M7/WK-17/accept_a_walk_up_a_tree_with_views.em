# Clause WK-17.1: "A parent link is an ordinary `weakref` field that names the object owning its
# holder (`parent: weakref Node`), which the program gives its target like any field: `child.parent =
# weakref this`, then `this.children.push(owns child)`. A read of one is compiled with no header test
# (a plain load and a test for no link) where the compiler proves its target lives"; "A walk up a
# tree with views (`cur = ref n`, `while cur.parent exists as p:`, `cur = ref p`) is then compiled
# with no test per step"
#$ test: run-pass
#$ rules: WK-17, WK-11, BCK-1
#$ stdout: leaf mid root
#$ stdout: 0
# Both conditions hold: every line that gives `parent` a target is followed by the line that puts
# the child into its parent's list, the nodes have one owner and no `drop`, and nothing in the walk
# can end a `Node`. That the reads carry no header test is a fact of the code, not of the output: the
# builder adds the code check (Q1).

class Node:
    name: String
    children: Array[Node] = []
    parent: weakref Node = None

    fn init(owns name: String):
        this.name = name

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(owns child)

fn path_up(n: Node) -> String:
    out = copy of n.name
    cur = ref n
    while cur.parent exists as p:
        out = f"{out} {p.name}"
        cur = ref p
    return out

fn main():
    root = Node(String("root"))
    mid = Node(String("mid"))
    mid.add(owns Node(String("leaf")))
    root.add(owns mid)
    println(path_up(root.children[0].children[0]))
    c = root.children.pop().unwrap()
    c.parent = None
    println(root.children.len())
