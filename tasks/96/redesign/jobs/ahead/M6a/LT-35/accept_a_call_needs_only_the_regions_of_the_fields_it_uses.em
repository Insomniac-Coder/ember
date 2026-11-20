# Clause LT-35.1: "For each function that can receive a view value, the compiler knows which fields
# the body may read, write, hand over, return or publish. A call requires only the regions of those
# fields."
#$ test: run-pass
#$ rules: LT-35, LT-36
#$ stdout: 7
#$ stdout: 7
# After `right` is pushed to, a direct call and a call through a function value that only ever
# holds `left_of` need only `p.left`'s region.

struct Pair:
    left: Span[int]
    right: Span[int]

fn left_of(p: Pair) -> int:
    return p.left[0]

fn main():
    left = [7]
    right = [8]
    p = Pair(left=left, right=right)
    right.push(owns 9)
    println(left_of(p))
    f: fn(Pair) -> int = left_of
    println(f(p))
