#$ test: compile-fail
#$ rules: RC-1, OWN-13, OBJ-5
# `[RC-1]` — `shares this` and `shares mut this` are refused in every method (`E2122`): the object
# may have one owner, which ends it with no count to test; the caller gives an owner instead.

class Node:
    v: int = 0

    fn enrol():
        keep = shares mut this        #$ error[E2122]: no second owner of a `Node` is made from inside its own method, since this `Node` may have one owner; let the caller give one: write `static fn enrol(owns me: shares mut Node):` and call it `Node.enrol(shares mut n)`
        println(keep.v)

fn main():
    n = Node()
    n.enrol()
