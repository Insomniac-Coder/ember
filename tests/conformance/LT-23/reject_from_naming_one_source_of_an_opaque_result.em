#$ test: compile-fail
#$ rules: LT-23, LT-22, LT-1a
## `[LT-23]` — with `from`, every field of the result points into what it
## names. Through an opaque function value each field may point into either
## argument (`chosen` may hold either function, ODR-278, so it is their shape),
## so `from left` cannot hold: the field from `right` is `E3062`.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn apply(f: fn(Span[i32], Span[i32]) -> Pair,
         left: Span[i32], right: Span[i32]) -> Pair from left:
    return f(left, right) #$ error[E3062]

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn crossed(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(right, left)

fn main():
    a: Array[i32] = Array[i32]()
    b: Array[i32] = Array[i32]()
    chosen = bundle if a.len() == 0 else crossed
    pair = apply(chosen, a.as_span(), b.as_span())
    println(pair.left.len())
