#$ test: compile-fail
#$ rules: UNS-5, ARN-11
# `[UNS-5]` — `zeroed[T]()` requires `T: needs Zeroable`: a class handle is never null, so zero bytes
# are no `Node`.

from std.mem import zeroed

class Node:
    value: int

fn main():
    n: Node = zeroed[Node]()    #$ error[E2040]: `Node` does not implement `std.mem.Zeroable`, which `T` requires
    println(n.value)
