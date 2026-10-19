#$ test: run-pass
#$ rules: TYP-14
#$ stdout: 3 5
# `[TYP-14]` — a `Box[T]` given to a borrowed `T` parameter is read through, so a recursive enum's
# `Box[Tree]` payload is traversed by a function taking `Tree` (D-733: it was `E2020`, "expected
# `Tree`, found `Box[Tree]`").

enum Tree:
    Leaf(int)
    Pair(Box[Tree], Box[Tree])

fn leaves(t: Tree) -> int:
    match t:
        Leaf(_):
            return 1
        Pair(a, b):
            return leaves(a) + leaves(b)

fn total(t: Tree) -> int:
    match t:
        Leaf(n):
            return n
        Pair(a, b):
            return total(a) + total(b)

fn main():
    t = Tree.Pair(Box(Tree.Leaf(1)), Box(Tree.Pair(Box(Tree.Leaf(2)), Box(Tree.Leaf(2)))))
    println(leaves(t), total(t))
