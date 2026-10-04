#$ test: run-pass
#$ rules: WK-15, WK-8, WK-1
#$ profiles: debug
#$ warning[L3001]: potential reference cycle: Node.next -> Node
#$ stdout: built
#$ stderr: warning[L3017]: reference cycle detected
#$ stderr: runtime ownership cycle:
#$ stderr: strong edges: 2
# `[WK-15]` — `ember run` in the debug profile reports the leaked objects and
# their cycles when the program ends, with no flag (`--no-leak-check` turns it
# off; the milestones check that and the other profiles).

class Node:
    name: String
    next: Option[Node]

fn main():
    a = Node("a", None)
    b = Node("b", Some(a))
    a.next = Some(b)
    println("built")
