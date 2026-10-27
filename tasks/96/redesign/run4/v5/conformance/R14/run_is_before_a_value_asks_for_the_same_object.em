#$ test: run-pass
#$ rules: TYP-44
#$ stdout: true false
#$ stdout: true
## design rules: R14
## R14: "`is` before a type name means "is of this kind" ...; before a value it means "the same
## object" (`a is b`, `x is None`)".

class Node:
    v: int = 0

fn main():
    a = shares Node(v=1)
    b = shares a
    c = shares Node(v=1)
    println(a is b, a is c)
    w: Option[int] = None
    println(w is None)
