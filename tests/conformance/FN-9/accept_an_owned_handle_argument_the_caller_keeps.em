#$ test: run-pass
#$ rules: FN-9, OWN-7, RC-1, OWN-6
#$ stdout: took 1
#$ after 1
#$ took 2
#$ drop 2
#$ after the temporary
#$ took 3
#$ drop 3
#$ after mem.drop
#$ drop 1
# `[FN-9]` (ODR-051, SP-010) — an `owned` handle argument read from a
# variable is a retained copy: the callee's release never frees the object,
# and the variable keeps it to its own end (`a`, dropped last). A handle made
# for the call is moved in and dies with the callee (`Node(2)`). `mem.drop`
# ends a variable's own handle where it is written (`c`, D-347).

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
    take(Node(2))
    println("after the temporary")
    c = Node(3)
    take(c)
    mem.drop(c)
    println("after mem.drop")
