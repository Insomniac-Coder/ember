#$ test: run-pass
#$ rules: RC-3, RC-1
#$ stdout: cleared
#$ 7
#$ drop 7
#$ end
# `[RC-3]` — the list is cleared while `t` holds its element, so `t` keeps a
# count of its own: the object lives until `t` ends.

class Node:
    v: int

    fn drop(mut self):
        println("drop", self.v)

fn main():
    nodes: Array[Node] = []
    nodes.push(Node(7))
    t = nodes[0]
    nodes.clear()
    println("cleared")
    println(t.v)
    mem.drop(t)
    println("end")
