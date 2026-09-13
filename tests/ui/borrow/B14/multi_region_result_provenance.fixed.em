@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn apply(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    pair = apply(left.as_span(), right.as_span())
    println(pair.left.len())
