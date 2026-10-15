#$ test: compile-fail
#$ rules: EFF-1, EFF-6
#$ error[E4041]: drops_one -> Node.drop -> console output
#$ error[E4041]: recursive -> count_up -> count_down -> console output
# `[EFF-1]` — a function's effects are its body's and its callees', over the call graph: the
# drops it performs run `Node.drop`, which prints, and a recursive group has the union of its
# members' effects.

class Node:
    v: int

    fn drop(mut self):
        println("drop", self.v)

@noio
fn drops_one(n: int) -> int:
    node = Node(n)
    return node.v

fn count_down(n: int) -> int:
    if n == 0:
        println("bottom")
        return 0
    return count_up(n - 1)

fn count_up(n: int) -> int:
    return count_down(n)

@noio
fn recursive(n: int) -> int:
    return count_up(n)

fn main():
    println(drops_one(1), recursive(2))
