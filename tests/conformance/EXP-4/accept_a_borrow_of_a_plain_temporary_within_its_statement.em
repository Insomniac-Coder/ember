#$ test: run-pass
#$ rules: EXP-4, BRW-9
#$ stdout: 3 6
# `[EXP-4]` (D-342) — within its statement a temporary may be borrowed
# freely, and a value copied out of the borrow outlives it.

struct W3:
    item: i64

fn inner(w: W3) -> ref i64:
    return ref w.item

fn sum(xs: Span[int]) -> int:
    total = 0
    for x in xs:
        total += x
    return total

fn main():
    n = inner(W3(3)) + 0
    println(n, sum([1, 2, 3]))
