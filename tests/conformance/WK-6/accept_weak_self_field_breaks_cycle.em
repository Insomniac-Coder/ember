#$ test: run-pass
#$ rules: WK-5, WK-6, WK-10, OBJ-3, WK-11, WK-12, TST-14
#$ profiles: debug, release, shipping
#$ stdout: 6

# The direct self-edge shape is accepted when its back-reference is weak. It
# must not contribute a strong one-node SCC or receive L3001.
class Node:
    value: i32 = 6
    next: Weak[Node] = Weak[Node].empty()

fn main():
    node = Node()
    println(node.value)
