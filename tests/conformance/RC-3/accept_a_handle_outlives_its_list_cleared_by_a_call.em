#$ test: run-pass
#$ rules: RC-3, RC-1, FN-9
#$ stdout: 3
#$ drop 3
#$ end
# `[RC-3]` — a call given the list mutably clears it while `t` lives, so `t`
# keeps a count of its own: the object lives until `t` ends.

class Node:
    v: int

    fn drop():
        println("drop", this.v)

fn wipe(ref mut xs: Array[Node]):
    xs.clear()

fn main():
    nodes: Array[Node] = []
    nodes.push(owns Node(3))
    t = nodes[0]
    wipe(ref mut nodes)
    println(t.v)
    mem.drop(t)
    println("end")
