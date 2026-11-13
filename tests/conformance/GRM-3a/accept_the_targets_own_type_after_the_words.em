#$ test: compile-pass
#$ rules: GRM-3a
## `[GRM-3a]` — what the refused lines of this directory write is the target's own type, held with the
## words `[TYP-45]` gives; the link's own accept form (`parent: weakref Node = None`,
## `other: weakref thread_shared Node = None`) is built with the `Link` kinds (M4: `E0900` until then).

class Node:
    value: int = 0

fn main():
    n = Node(value=1)
    println(n.value)
