#$ test: run-pass
#$ rules: WK-11, WK-12
#$ stdout: true true true true
# `[WK-11]` — a link may hold no target: `None` is a `weakref T` that holds none wherever the place's type
# says it is a link, in a typed local, a return, a field default and an argument (it was `Weak.empty()`,
# whose type the place gave, D-401), and each opens to `else:`.

class Node:
    v: int
    parent: weakref Node = None

fn orphan() -> weakref Node:
    return None

fn gone(w: weakref Node) -> bool:
    if w exists as _:
        return false
    return true

fn main():
    e: weakref Node = None
    n = Node(5)
    println(gone(e), gone(orphan()), gone(n.parent), gone(None))
