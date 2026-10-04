#$ test: run-pass
#$ rules: RC-2d, RC-2, OWN-5, RC-1
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: drop 5
#$ stdout: 7
#$ stdout: drop 1
#$ stdout: 8
#$ stdout: done
#$ stdout: drop 8
#$ stdout: drop 7
# `[RC-2d]` — a handle stored into a field (or an element) from a temporary
# moves there: no retain, and the temporary's end releases nothing. The old
# value is still dropped first (`[OWN-5]`). The store used to retain the new
# handle and release it again at the statement's end (D-505).

class Leaf:
    v: int

    fn drop(mut self):
        println("drop", self.v)

class Tree:
    leaf: Leaf

fn make(v: int) -> Leaf:
    return Leaf(v)

fn main():
    t = Tree(Leaf(5))
    t.leaf = Leaf(7)
    println(t.leaf.v)
    leaves: Array[Leaf] = [Leaf(1)]
    leaves[0] = make(8)
    println(leaves[0].v)
    println("done")
