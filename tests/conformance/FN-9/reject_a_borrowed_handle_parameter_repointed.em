#$ test: compile-fail
#$ rules: FN-9, FN-1
#$ error[E3023]: `n` is the node the caller passed and stays that node for the whole call; for a node of your own write `new_n = Node(2)` in place of line 11
# `[FN-9]` — what a borrowed handle parameter may not do is re-point the caller's handle: that is
# a `mut` parameter's.

class Node:
    v: int

fn repoint(n: Node):
    n = Node(2)

fn main():
    n = Node(1)
    repoint(n)
    println(n.v)
