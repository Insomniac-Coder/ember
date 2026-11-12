#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26, OBJ-3
#$ profiles: debug, development, release
#$ stdout: 1
#$ assert-c: contains(ember_obj_header* value)
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_release)

struct Node:
    weak: weakref Node

    fn drop():
        # The final strong release has started deinitialisation, so this
        # upgrade must fail rather than resurrecting the payload.
        if this.weak exists as _:
            println(0)
        else:
            println(1)

fn main():
    node = shares mut Node(None)
    node.weak = weakref node
