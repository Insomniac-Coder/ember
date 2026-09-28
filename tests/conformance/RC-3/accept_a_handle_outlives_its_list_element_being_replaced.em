#$ test: run-pass
#$ rules: RC-3, RC-1
#$ stdout: 1
#$ drop 1
#$ end
#$ drop 5
# `[RC-3]` — the element `t` was copied from is replaced while `t` lives, so
# `t` keeps a count of its own: the old object lives until `t` ends.

class Node:
    v: int

    fn drop(mut self):
        println("drop", self.v)

fn main():
    nodes: Array[Node] = []
    nodes.push(Node(1))
    t = nodes[0]
    nodes[0] = Node(5)
    println(t.v)
    mem.drop(t)
    println("end")
