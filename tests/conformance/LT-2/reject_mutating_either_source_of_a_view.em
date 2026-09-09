#$ test: compile-fail
#$ rules: LT-2, LT-1, BRW-1
# The intersection, biting. `v` points only into `p` — `pick` returns its
# first argument — and mutating `q` is still rejected, because the result's
# region is the intersection of both parameters' and the compiler will not
# guess which one the body picked.
#
# This is also why `E3064` is unreachable in v1. `[LT-2]` says the
# intersection is *taken*, so construction never fails; the shape that needs
# `E3064` is a view struct whose fields must have **different** regions, and
# v1 has no way to ask for that — region syntax is reserved for v2 (`[LT-6]`,
# `E0007`) and `[TYP-15a]`'s `BorrowList`/`ViewList` are not implemented.

fn pick(a: Span[i32], b: Span[i32]) -> Span[i32]:
    return a

fn main():
    p: Array[i32] = Array[i32]()
    p.push(1)
    q: Array[i32] = Array[i32]()
    q.push(2)
    v = pick(p, q)
    q.push(3)              #$ error[E3021]: `q` is borrowed here and mutably borrowed elsewhere
    println(v[0])
