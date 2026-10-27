#$ test: run-pass
#$ rules: R12, R8, R10
#$ stdout: a
#$ stdout: b
## Step 3's hole 3, part 1, R12 (rules.md lines 681-687): `s`, a view into the node's text reached
## through the link, is used before `rename()` replaces that text, so it does not live across the call;
## the link opened again afterwards reads the new name, which the function's own `n` changed. The nearest
## allowed program to reject_a_view_through_a_link_used_after_a_stored_function_replaces_its_text.

class Node:
    name: String

fn main():
    n = Node(name=String(" a "))
    w = weakref n
    rename = fn() owns n: n.name = String("b")
    if w exists as p:
        s = p.name.trim()
        println(s)
        rename()
    if w exists as q:
        println(q.name)
