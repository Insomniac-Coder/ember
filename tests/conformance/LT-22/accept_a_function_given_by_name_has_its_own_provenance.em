#$ test: run-pass
#$ rules: LT-22, FN-6, CLO-3
#$ profiles: debug, development, release
#$ stdout: 2 3
# `[LT-22]`, ODR-278 — a function given by name to a callable parameter is that function, by its own
# type, and the parameter's instance calls it directly (`[CLO-3]`), so the result's provenance is
# the function's summary: each field from its own parameter. Through a value that may hold either
# of two functions it is unknown (`reject_opaque_multi_region_result_provenance.em`).

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn apply(f: fn(Span[i32], Span[i32]) -> Pair,
         left: Span[i32], right: Span[i32]) -> Pair:
    return f(left, right)

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn main():
    left: Array[i32] = [1, 2]
    right: Array[i32] = [3, 4, 5]
    pair = apply(bundle, left.as_span(), right.as_span())
    println(pair.left.len(), pair.right.len())
