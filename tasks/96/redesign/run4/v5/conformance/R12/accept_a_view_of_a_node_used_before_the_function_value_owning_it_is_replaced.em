#$ test: run-pass
#$ rules: BRW-17, LT-3
#$ stdout: a
#$ stdout: gone
## design rules: R12, R10, R11, R13
## Step 3's hole 3, part 2, R12 (rules.md lines 687-691): `p` is used before `h.keep = fn(): pass` ends
## the function value and the node its state owned, so it does not live across that line; the link
## opened again afterwards finds the node ended. The nearest allowed program to
## reject_a_view_of_a_node_used_after_the_function_value_owning_it_is_replaced.

class Node:
    name: String

class Holder:
    keep: fn()

fn main():
    n = Node(name=String("a"))
    w = weakref n
    h = Holder(keep=fn() owns n: println(n.name))
    if w exists as p:
        println(p.name)
        h.keep = fn(): pass
    if w exists as q:
        println(q.name)
    else:
        println("gone")
