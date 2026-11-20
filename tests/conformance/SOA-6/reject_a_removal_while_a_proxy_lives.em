#$ test: compile-fail
#$ rules: SOA-6, BRW-16
# `[SOA-6]` — a proxy is a position: adding to `ps` keeps it, and a removal, reorder or clear ends it, so
# a later use is `E3060`.

struct P:
    a: int
    b: int

fn main():
    ps = SoA[P].with_capacity(2)
    ps.push(owns P(1, 2))
    ps.push(owns P(5, 6))
    p = ref mut ps[0]
    ps.clear()
    p.a = 3    #$ error[E3060]
    q = ps[0]
    _gone = ps.swap_remove(0)
    println(q.a)    #$ error[E3060]
