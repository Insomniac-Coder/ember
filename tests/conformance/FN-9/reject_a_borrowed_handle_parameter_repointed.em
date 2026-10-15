#$ test: compile-fail
#$ rules: FN-9, FN-1
#$ error[E3023]: cannot mutate borrowed parameter `n`
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
