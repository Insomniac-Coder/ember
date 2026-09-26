#$ test: run-pass
#$ rules: FN-9, OWN-7, RC-1
#$ stdout: took 1
#$ after 1
#$ took 2
#$ drop 2
#$ drop 1
# `[FN-9]` (ODR-051, SP-010) — an `owned` handle argument is the caller's own
# handle, moved, when the call is its last use, and otherwise a retained copy,
# so the callee's release never frees an object the caller still reads: `a`
# is used after the call and outlives it; `b` is not, and dies in `take`.

class Node:
    v: int

    fn drop(mut self):
        println("drop", self.v)

fn take(owned n: Node):
    println("took", n.v)

fn main():
    a = Node(1)
    take(a)
    println("after", a.v)
    b = Node(2)
    take(b)
