# Clause WK-3.1: "An object is ended the moment its last owner lets it go, before its `drop` runs
# (...): a `weakref` link opened during its `drop`, or during the `drop` of anything it owns (a
# child's `drop` opening `this.parent` while the parent is being ended), finds it ended (...), so
# the `if ... exists as p:` block is skipped and the half-ended object is never read, nor counted
# again"; "Ending the link itself (`this.parent = None`) stays allowed"
#$ test: run-pass
#$ rules: WK-3, WK-2, OBJ-5, CLS-6
#$ stdout: leaf sees root
#$ stdout: root sees itself ended
#$ stdout: leaf2 finds its parent ended
# `leaf` is removed while `root` lives, so its `drop` finds its parent; `root` ends with `main`: its
# own `drop` opens a link to itself and finds it ended, and `leaf2`'s `drop`, run as `root`'s
# fields end, finds its parent ended. Each `drop` then ends its own link. The `thread_shared` case,
# ended at its making thread's next check point, is M9's: not here.

class Node:
    name: String
    children: Array[Node] = []
    parent: weakref Node = None
    me: weakref Node = None

    fn init(owns name: String):
        this.name = name

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(owns child)

    fn drop():
        if this.parent exists as p:
            println(f"{this.name} sees {p.name}")
        elif this.me exists as m:
            println(f"{this.name} sees {m.name}")
        elif this.children.len() > 0:
            println(f"{this.name} sees itself ended")
        else:
            println(f"{this.name} finds its parent ended")
        this.parent = None

fn main():
    root = Node(String("root"))
    root.me = weakref root
    root.add(owns Node(String("leaf")))
    root.add(owns Node(String("leaf2")))
    root.children.remove(0)
