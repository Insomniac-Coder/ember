#$ test: run-pass
#$ rules: RC-3, RC-1, DRP-2
#$ stdout: made
#$ after
#$ kept 2
#$ end of main
#$ drop 2
#$ drop 1
# `[RC-3]` (ODR-063, SP-014) — an object is deinitialised when its last strong
# owner ends as the source says, never earlier: `_a` is never read after it is
# bound, yet it lives to the end of `main`; the object behind `b` outlives
# `b`'s reassignment because the array still holds it. Owners end in reverse
# order of declaration.

class Node:
    v: int

    fn drop(mut self):
        println("drop", self.v)

fn main():
    _a = Node(1)
    println("made")
    println("after")
    b = Node(2)
    kept = [b]
    println("kept", kept[0].v)
    println("end of main")
