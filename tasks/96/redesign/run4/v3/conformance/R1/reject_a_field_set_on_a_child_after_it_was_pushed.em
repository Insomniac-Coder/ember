#$ test: compile-fail
#$ rules: R1, R2
#$ help: set it before the push (`child.value = 5` above line 15), or change it in the list: `root.children[0].value = 5`; for two owners that both change it write `root.children.push(shares mut child)` with `children: Array[shares mut Node]`
## R1, R2: `root.children.push(child)` hands `child` over to the list (a value stored somewhere else),
## so a later write through `child` is a use after the hand-over (E3040), not a silent write to a dead
## local (AN-36: today this compiles and prints 1).

struct Node:
    value: int
    children: Array[Node] = []

fn main():
    root = Node(value=0)
    child = Node(value=1)
    root.children.push(child)
    child.value = 5            #$ error[E3040]: `child` was handed over to `root.children` on line 15 and is changed on line 16, which would change nothing
    println(root.children[0].value)
