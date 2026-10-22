#$ test: run-pass
#$ rules: STR-5, TYP-36
#$ profiles: debug, development, release
#$ stdout: 7 7
#$ 2 1
#$ 10 6
#$ 1 3
## IX.1, `[STR-5]` (ODR-237) — `Box[T]` is `Clone` when `T` is: `b.clone()` is a new box holding a
## clone of the value (the box's own `clone`, before auto-deref's), so a struct or enum with a
## `Box` field derives `Clone`, a recursive one included: its clone is a function that calls
## itself. Each clone is its own: changing the original after leaves it as it was.

struct Point:
    x: int
    label: String

struct Holder:
    p: Box[Point]
    n: int

enum Tree:
    Leaf(int)
    Node(Box[Tree], Box[Tree])

fn total(t: Tree) -> int:
    match t:
        Tree.Leaf(n):
            return n
        Tree.Node(left, right):
            return total(left) + total(right)

fn main():
    b = Box(Point(7, "a"))
    c: Box[Point] = b.clone()
    println(b.x, c.x)
    h = Holder(Box(Point(1, "b")), 1)
    k = h.clone()
    h.p.x = 2
    println(h.p.x, k.p.x)
    t = Tree.Node(Box(Tree.Leaf(1)), Box(Tree.Node(Box(Tree.Leaf(2)), Box(Tree.Leaf(3)))))
    u = t.clone()
    t = Tree.Leaf(10)
    println(total(t), total(u))
    nested: Box[Box[Point]] = Box(Box(Point(1, "c")))
    copy = nested.clone()
    nested.x = 3
    println(copy.x, nested.x)
