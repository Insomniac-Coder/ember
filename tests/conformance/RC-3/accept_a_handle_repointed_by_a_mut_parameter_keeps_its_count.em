#$ test: run-pass
#$ rules: RC-3, RC-1, FN-9
#$ stdout: 9
#$ drop 9
#$ end
#$ drop 1
# `[RC-3]`, `[FN-9]` — a `mut` handle parameter may re-point `t`, so `t`
# keeps a count of its own: the replacement is dropped when `t` ends and the
# original only with the list.

class Node:
    v: int

    fn drop():
        println("drop", this.v)

fn replace(ref mut h: shares Node):
    h = shares Node(9)

fn main():
    nodes: Array[shares Node] = []
    nodes.push(owns shares Node(1))
    t = shares nodes[0]
    replace(ref mut t)
    println(t.v)
    mem.drop(t)
    println("end")
