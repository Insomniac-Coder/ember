#$ test: compile-fail
#$ rules: LT-23, LT-1a, BRW-1
## `[LT-23]` — under `@borrows(left, right)` the field built from `right`
## keeps borrowing `b`, so changing `b` while that field is live is refused.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

@borrows(left, right)
fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn main():
    a: Array[i32] = Array[i32]()
    b: Array[i32] = Array[i32]()
    pair = bundle(a.as_span(), b.as_span())
    b.push(2) #$ error[E3021]: `b` is borrowed here and mutably borrowed elsewhere
    println(pair.right.len())
