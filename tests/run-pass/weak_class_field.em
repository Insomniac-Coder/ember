#$ test: run-pass
#$ rules: OBJ-3, WK-1, WK-2, WK-3, WK-11, WK-12, WK-14
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_release)

# A weak field is a non-owning class back-reference. The empty default also
# exercises a self-referential `Weak[Node]` C declaration before any instance
# is constructed.
class Node:
    value: i32 = 7
    parent: Weak[Node] = Weak[Node].empty()

fn main():
    node = Node()
    match node.parent.upgrade():
        Some(_) => println(0)
        None => println(node.value)
