#$ test: compile-fail
#$ rules: SOA-6, BRW-1
#$ error[E3021]: cannot write through a shared reference
# `[SOA-6]` — an `SoARef` borrows its `SoA` shared: its fields read and do not write.

struct P:
    a: int
    b: int

fn reset(p: SoARef[P]):
    p.a = 0

fn main():
    ps = SoA[P].with_capacity(2)
    ps.push(P(1, 2))
    reset(ps[0])
