#$ test: run-pass
#$ rules: LT-22, LT-35, LT-41
#$ stdout: 0
# `[LT-22]` — through a function value the field-to-source relation is unknown, so every field of
# the result points into every source argument (`[LT-1]` rule 3): `pair` keeps `left` and `right`
# viewed. (0.9.10 refused it, `E3065`.)

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    make: fn(Span[i32], Span[i32]) -> Pair = bundle
    pair = make(left.as_span(), right.as_span())
    println(pair.left.len())
