#$ test: run-pass
#$ rules: R1, R2
#$ stdout: 6
## R1, R2: the nearest program that is allowed: the child is set before it is handed over, and
## changed afterwards through the list that owns it (AN-36's first NEW form).

struct Node:
    value: int
    children: Array[Node] = []

fn main():
    root = Node(value=0)
    child = Node(value=1)
    child.value = 5
    root.children.push(child)
    root.children[0].value = 6
    println(root.children[0].value)
