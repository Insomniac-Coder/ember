#$ test: run-pass
#$ rules: RC-3, RC-1, FN-9
#$ stdout: 3
#$ drop 3
#$ end
# `[RC-3]` — a call given the list mutably clears it while `t` lives, so `t`
# keeps a count of its own: the object lives until `t` ends.

class Node:
    v: int

    fn drop(mut self):
        println("drop", self.v)

fn wipe(mut xs: Array[Node]):
    xs.clear()

fn main():
    nodes: Array[Node] = []
    nodes.push(Node(3))
    t = nodes[0]
    wipe(nodes)
    println(t.v)
    mem.drop(t)
    println("end")
