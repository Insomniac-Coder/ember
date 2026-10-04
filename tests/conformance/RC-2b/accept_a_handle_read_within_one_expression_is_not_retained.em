#$ test: run-pass
#$ rules: RC-2b, RC-2, RC-1
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 30
#$ stdout: 2
# `[RC-2b]` — a handle read from a place (a field, an element) and used only
# within one expression, while nothing writes that place, is not retained.

class Leaf:
    v: int

class Tree:
    leaf: Leaf

fn main():
    t = Tree(Leaf(5))
    leaves: Array[Leaf] = [Leaf(1)]
    total = 0
    for i in 0..3:
        total += t.leaf.v + t.leaf.v
    println(total)
    println(leaves[0].v + leaves[0].v)
