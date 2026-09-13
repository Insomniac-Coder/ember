#$ test: compile-fail
#$ rules: LT-22, LT-35, LT-41, DIA-19, TST-19

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
    pair = make(left.as_span(), right.as_span()) #$ error[E3065]: the returned `Pair` has field provenance that cannot be inferred soundly
    println(pair.left.len())
