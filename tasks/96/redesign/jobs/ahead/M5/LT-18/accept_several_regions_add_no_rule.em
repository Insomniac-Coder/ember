# Clause LT-18.2: "multiple regions add no new aliasing rule and weaken none."
#$ test: run-pass
#$ rules: LT-18, LT-24, LT-34, BRW-2
#$ stdout: 1
#$ stdout: 2
#$ stdout: [1, 3]
#$ stdout: 4
# After `pair.left`'s last use, `a` may change while `pair.right` is still used: each field's view
# ends at its own last use, as two separate views would (`[LT-24]`). Two fields viewing one list read
# it together, as any two read-only views may.

@view
struct Pair:
    left: Span[int]
    right: Span[int]

fn main():
    a = [1]
    b = [2]
    pair = Pair(left=a.as_span(), right=b.as_span())
    println(pair.left[0])
    a.push(owns 3)
    println(pair.right[0])
    println(a)
    both = Pair(left=a[0..1], right=a[1..])
    println(both.left[0] + both.right[0])
