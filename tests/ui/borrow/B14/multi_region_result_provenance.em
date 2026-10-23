@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn apply(f: fn(Span[i32], Span[i32]) -> Pair,
         left: Span[i32], right: Span[i32]) -> Pair:
    return f(left, right)

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn crossed(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(right, left)

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    chosen = bundle if left.len() == 0 else crossed
    pair = apply(chosen, left.as_span(), right.as_span())
    println(pair.left.len())
