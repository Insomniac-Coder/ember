#$ test: run-pass
#$ rules: RC-2a, RC-2
#$ profiles: development, release
#$ stdout: 1 6
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
# `[RC-2a]` — a parameter that is not `owns` reads the caller's shared value: no count is taken, of an
# object or of a value in a block.

class Node:
    v: int = 1

@noinline
fn of_node(x: shares Node) -> int:
    return x.v

@noinline
fn of_value(x: shares int) -> int:
    return x + 1

fn main():
    n = shares Node()
    k = shares 5
    println(of_node(n), of_value(k))
