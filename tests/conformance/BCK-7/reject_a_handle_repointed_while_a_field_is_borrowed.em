#$ test: compile-fail
#$ rules: BCK-7, RC-5
#$ error[E3080]: `n.v` cannot be written while it is borrowed
# `[BCK-7]` — a borrow of a field reached through a handle is a loan of that path: re-pointing the
# handle writes the path's prefix while the loan is live.

class Node:
    v: Array[int]

fn main():
    n = Node([1, 2])
    r: ref Array[int] = ref n.v
    n = Node([3])
    println(r[0])
