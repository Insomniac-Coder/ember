#$ test: run-pass
#$ rules: LT-23, LT-1a, LT-24
#$ stdout: 1
#$ stdout: 0
## `[LT-23]` — `@borrows(left, right)` names both sources, but each field of
## the result still borrows only the one it was built from: the caller may
## change `b` while it uses only the field that borrows `a`, whichever position
## the source was passed in.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

@borrows(left, right)
fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

@borrows(left, right)
fn swapped(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(right, left)

fn main():
    a: Array[i32] = Array[i32]()
    b: Array[i32] = Array[i32]()
    a.push(1)
    pair = bundle(a.as_span(), b.as_span())
    b.push(2)
    println(pair.left.len())
    other = swapped(a.as_span(), b.as_span())
    a.push(3)
    println(other.left.len() - 1)
