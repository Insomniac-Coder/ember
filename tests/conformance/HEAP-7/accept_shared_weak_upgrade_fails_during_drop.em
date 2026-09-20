#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26, OBJ-3
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ assert-c: contains(ember_weak_upgrade)
#$ assert-c: contains(ember_weak_retain)
#$ assert-c: contains(ember_weak_release)

struct Node:
    weak: Weak[Shared[Node]]

    fn drop(mut self):
        # The final strong release has started deinitialisation, so this
        # upgrade must fail rather than resurrecting the payload.
        match self.weak.upgrade():
            Some(_):
                println(0)
            None:
                println(1)

fn main():
    node = Shared(Node(Weak[Shared[Node]].empty()))
    alias = node
    payload = node.get_mut()
    payload.weak = Weak(alias)
