#$ test: compile-fail
#$ rules: SOA-6, BRW-1
#$ error[E3022]: `ps` is already mutably borrowed
# `[SOA-6]` — an `SoAMut` borrows all of its `SoA` mutably for the proxy's life, so `push` while one
# is used later conflicts.

struct P:
    a: int
    b: int

fn main():
    ps = SoA[P].with_capacity(2)
    ps.push(P(1, 2))
    p = ps[0]
    ps.push(P(3, 4))
    println(p.a)
