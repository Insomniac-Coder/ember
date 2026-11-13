#$ test: compile-fail
#$ rules: SOA-2, FN-1
#$ error[E3023]: `ps[0].a = 5` writes an element of the caller's list, but `ps` only reads it; write `fn bump(ref mut ps: SoA[P]):` and call it `bump(ref mut ps)`
# `[SOA-2]` — a column is a `MutSpan` only where its `SoA` is mutable: a borrowed parameter's is not.

struct P:
    a: int
    b: int

fn bump(ps: SoA[P]):
    ps[0].a = 5

fn main():
    ps = SoA[P].with_capacity(4)
    ps.push(owns P(1, 2))
    bump(ps)
