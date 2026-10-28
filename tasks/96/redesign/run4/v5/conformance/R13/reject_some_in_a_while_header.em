#$ test: compile-fail
#$ rules: GRM-44
#$ help: write `while cur.next exists as n:`
## design rules: R13
## R13: the same for a `while` header (E0115; AN-63's message).

class Node:
    value: int
    next: Option[Node] = None

fn main():
    head = Node(value=1, next=Some(Node(value=2)))
    cur = ref head
    while Some(n) = cur.next:  #$ error[E0115]: an `Option` or a `weakref` link is opened with `exists as`
        cur = ref n
    println(cur.value)
