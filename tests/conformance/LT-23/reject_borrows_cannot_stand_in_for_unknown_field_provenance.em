#$ test: compile-fail
#$ rules: LT-23, LT-22, LT-1a
## `[LT-23]` — `@borrows(left, right)` says which parameters the result may
## borrow, not which field borrows which: through an opaque function value the
## per-field relation is unknown, and naming both sources does not collapse the
## two regions into one to make up for it.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

@borrows(left, right)
fn apply(f: fn(Span[i32], Span[i32]) -> Pair,
         left: Span[i32], right: Span[i32]) -> Pair:
    return f(left, right) #$ error[E3065]: the returned `Pair` has field provenance that cannot be inferred soundly

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn main():
    a: Array[i32] = Array[i32]()
    b: Array[i32] = Array[i32]()
    pair = apply(bundle, a.as_span(), b.as_span())
    println(pair.left.len())
