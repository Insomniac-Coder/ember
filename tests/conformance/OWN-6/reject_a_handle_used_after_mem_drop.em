#$ test: compile-fail
#$ rules: OWN-6, OWN-3
# `[OWN-6]` — `mem.drop(x)` ends `x`'s ownership, a class handle's too, though
# a handle is `Copy`: it is moved, so a later use is `E3040` (D-347; before,
# it passed a retained copy and the object lived on).

class Node:
    v: int

fn main():
    a = Node(1)
    mem.drop(a)
    println(a.v) #$ error[E3040]: `a` was handed over on line 12 and is used on line 13; to keep it, write `owns copy of a`
