#$ test: run-pass
#$ rules: BCK-7
#$ stdout: 1 2 2
# `[BCK-7]` — reading through a handle, or copying it, while a field reached through it is
# borrowed takes no loan of the handle's local and conflicts with nothing; the borrow keeps the
# object alive for as long as it is used.

class Node:
    v: Array[int]

fn main():
    n = Node([1, 2])
    r: ref Array[int] = ref n.v
    m = n
    println(r[0], n.v.len(), m.v.len())
