#$ test: run-pass
#$ rules: STR-1, TYP-25, CLS-3
#$ stdout: 1 2 3
#$ stdout: 1 0 3
#$ stdout: 4 5
# `[TYP-25]` — positional arguments come first, then named ones, in a
# memberwise constructor as in a call; a field with a default may be omitted.

struct P:
    a: int
    b: int = 0
    c: int

class Q:
    x: int
    y: int

fn main():
    p = P(1, c = 3, b = 2)
    println(p.a, p.b, p.c)
    r = P(1, c = 3)
    println(r.a, r.b, r.c)
    q = Q(4, y = 5)
    println(q.x, q.y)
