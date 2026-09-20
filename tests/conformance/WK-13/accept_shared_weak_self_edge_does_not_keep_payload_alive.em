#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26, OBJ-3
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ stdout: 7
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_release)
#$ assert-c: contains(ember_release((ember_obj_header*)

struct Node:
    weak: Weak[Shared[Node]]
    value: i32

    fn drop(mut self):
        println(self.value)

fn main():
    node = Shared(Node(Weak[Shared[Node]].empty(), 7))
    alias = node
    payload = node.get_mut()
    payload.weak = Weak(alias)
    println(payload.value)
