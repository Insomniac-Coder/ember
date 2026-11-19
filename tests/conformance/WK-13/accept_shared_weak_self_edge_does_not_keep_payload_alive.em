#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26, OBJ-3
#$ profiles: debug, development, release
#$ stdout: 7
#$ stdout: 7
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_release)
#$ assert-c: contains(ember_release((ember_obj_header*)

struct Node:
    weak: weakref Node
    value: i32

    fn drop():
        println(this.value)

fn main():
    node = shares mut Node(None, 7)
    node.weak = weakref node
    println(node.value)
