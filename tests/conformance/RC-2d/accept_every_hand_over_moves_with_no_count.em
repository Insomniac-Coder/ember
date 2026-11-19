#$ test: run-pass
#$ rules: RC-2d, RC-2
#$ profiles: development, release
#$ stdout: 1
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
# `[RC-2d]` — an owner handed over moves, and nothing is counted: `b = a`, an `owns` argument, `return a`,
# and a value stored into a field from a temporary.

class Node:
    v: int = 1

class Holder:
    item: shares Node

@noinline
fn pass_on(owns n: shares Node) -> shares Node:
    return n

fn main():
    a = shares Node()
    b = a
    c = pass_on(owns b)
    h = Holder(c)
    h.item = shares Node()
    println(h.item.v)
