#$ test: run-pass
#$ rules: R1, R4
#$ stdout: 1
#$ 1
## R1, R4: the nearest program that is allowed: the object is read before the call that takes it
## over (AN-43's NEW).

class Node:
    v: int

fn take(owns n: Node) -> int:
    return n.v

fn main():
    a = Node(v=1)
    println(a.v)
    x = take(a)
    println(x)
