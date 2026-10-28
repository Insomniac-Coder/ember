#$ test: run-pass
#$ rules: FN-1, OWN-1
#$ stdout: 1 2
## design rules: R4, R1
## R4: `owns n: Node` takes the object over, and the hand-over is written as `=` writes one, with no
## word at the call; a number parameter is copied in (AN-43).

class Node:
    v: int

fn take(owns n: Node) -> int:
    return n.v

fn add_one(n: int) -> int:
    return n + 1

fn main():
    a = Node(v=1)
    x = take(a)
    b = 1
    y = add_one(b)
    println(x, y)
