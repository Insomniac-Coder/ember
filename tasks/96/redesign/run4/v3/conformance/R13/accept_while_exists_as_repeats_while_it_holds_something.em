#$ test: run-pass
#$ rules: R13, R2
#$ stdout: 5
## R13, R2: `while cur.next exists as n:` repeats while the `Option` holds a node, naming it `n` in
## each round; `cur = ref n` points the view at the next node (R2).

class Node:
    value: int
    next: Option[Node] = None

fn main():
    head = Node(value=1, next=Some(Node(value=2, next=Some(Node(value=3)))))
    total = 0
    cur = ref head
    while cur.next exists as n:
        total += n.value
        cur = ref n
    println(total)
