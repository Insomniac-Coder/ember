#$ test: compile-fail
#$ rules: LT-24, LT-35, LT-36, LT-42, BRW-1, TST-19

# Exact capture paths are a union, not a choice of the first path observed.
# This closure reads both fields, so the later mutation of `right` remains a
# conflict even though the first access is to `left`.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn main():
    left: Array[i32] = Array[i32]()
    left.push(7)
    right: Array[i32] = Array[i32]()
    right.push(8)
    pair = bundle(left.as_span(), right.as_span())
    total = fn() => pair.left[0] + pair.right[0]
    right.push(9) #$ error[E3021]: `right` is borrowed here and mutably borrowed elsewhere
    println(total())
