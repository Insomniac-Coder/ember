# Clause WK-5.2: "`weakref` links, `sharedref mut` links, views and raw pointers are not owner edges.
# A `let` field is an owner edge; it is the line that sets it, at the object's making, that closes no
# loop, since it can name only objects that exist already."
#$ test: run-pass
#$ rules: WK-5, WK-11, CLS-9
#$ stdout: a
#$ stdout: b
# A `let` owner set at the making names an object made before; a link back is a `weakref`.

class Node:
    name: String
    let peer: Option[shares mut Node]
    back: weakref Node = None

    fn init(owns name: String, owns peer: Option[shares mut Node]):
        this.name = name
        this.peer = peer

fn main():
    a = shares mut Node(String("a"), None)
    b = shares mut Node(String("b"), Some(shares mut a))
    a.back = weakref b
    if b.peer exists as p:
        println(p.name)
    if a.back exists as q:
        println(q.name)
