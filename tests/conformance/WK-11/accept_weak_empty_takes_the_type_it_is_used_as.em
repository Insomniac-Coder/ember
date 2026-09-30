#$ test: run-pass
#$ rules: WK-11
#$ stdout: true true true true
# D-401 — `[WK-11]`: "`Weak[O].empty()` (or `Weak.empty()` where the type is
# known) creates one that points nowhere". The bare form was `E1010`, cannot
# find `Weak`, in a typed local, a return, a field default and an argument.

class Node:
    v: int
    parent: Weak[Node] = Weak.empty()

fn orphan() -> Weak[Node]:
    return Weak.empty()

fn gone(w: Weak[Node]) -> bool:
    return w.upgrade().is_none()

fn main():
    e: Weak[Node] = Weak.empty()
    n = Node(5)
    println(e.upgrade().is_none(), orphan().upgrade().is_none(), n.parent.upgrade().is_none(), gone(Weak.empty()))
