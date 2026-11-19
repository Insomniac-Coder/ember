#$ test: run-pass
#$ rules: OBJ-3, WK-1, WK-2, WK-3, WK-11, WK-12, WK-14
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_release)

# A weak field is a non-owning class back-reference. The empty default also
# exercises a self-referential `weakref Node` C declaration before any instance
# is constructed.
class Node:
    value: i32 = 7
    parent: weakref Node = None

fn main():
    node = Node()
    if node.parent exists as _:
        println(0)
    else:
        println(node.value)
