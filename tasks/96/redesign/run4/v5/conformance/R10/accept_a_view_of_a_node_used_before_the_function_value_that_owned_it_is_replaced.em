#$ test: run-pass
#$ rules: R10, R12, R5
#$ stdout: 1
#$ stdout: gone
## R10 point 1, R12 (hole 3, part 2): the way out E3060's message gives: `p` is used before
## `h.keep = fn(): pass`, which ends the function stored there and the node it owns; the link opened
## again after it finds the node ended. The nearest program to
## `reject_a_view_of_a_node_used_after_the_function_value_that_owned_it_is_replaced.em` that is allowed.

class Node:
    v: int = 1

class Holder:
    keep: fn()

fn main():
    n = Node()
    w = weakref n
    h = Holder(keep=fn() owns n: println(n.v))
    if w exists as p:
        println(p.v)
        h.keep = fn(): pass
    if w exists as q:
        println(q.v)
    else:
        println("gone")
