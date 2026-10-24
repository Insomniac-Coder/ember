# A10: a recursive struct written the Python way, with no Box.
struct Tree:
    value: int
    left: Option[Tree] = None
    right: Option[Tree] = None

fn total(t: Tree) -> int:
    s = t.value
    match t.left:
        Some(l):
            s += total(l)
        None:
            pass
    return s

fn main():
    t = Tree(value=1, left=Some(Tree(value=2)))
    println(total(t))
