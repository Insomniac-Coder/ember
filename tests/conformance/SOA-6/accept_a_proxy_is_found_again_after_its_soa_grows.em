#$ test: run-pass
#$ rules: SOA-6, BRW-16
#$ stdout: 7 100
#$ stdout: 1
# `[SOA-6]` — a proxy is a position: it finds element `i` again at each use, so adding to `ps` (a hundred
# pushes move every column) keeps it; a read-only proxy the same.

struct P:
    a: int
    b: int

fn main():
    ps = SoA[P].with_capacity(1)
    ps.push(owns P(1, 2))
    p = ref mut ps[0]
    for k in 1..100:
        ps.push(owns P(k, k))
    p.a = 7
    println(ps[0].a, ps.len())
    q = ps[0]
    ps.push(owns P(0, 0))
    println(q.b - 1)
