#$ test: run-pass
#$ rules: R8, R10, R12
#$ stdout: a
## R8 (b), R12 (hole 3, part 1): the way out E3060's message gives: `s` is a `String` of your own, so
## `rename()`, which replaces the name of the node its state owns, does not end it. The nearest program
## to `reject_a_view_of_text_used_after_a_function_value_renames_its_node.em` that is allowed.

class Node:
    name: String

fn main():
    n = Node(name=String(" a "))
    w = weakref n
    rename = fn() owns n: n.name = String("b")
    if w exists as p:
        s = String(p.name.trim())
        rename()
        println(s)
