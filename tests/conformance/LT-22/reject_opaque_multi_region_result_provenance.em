#$ test: compile-fail
#$ rules: LT-22, LT-35, LT-41, DIA-19, TST-19
# `[LT-22]` — through an opaque function value the result's field provenance is unknown: `chosen`
# may hold either function (ODR-278: where two functions meet, the value is their shape, `fn(…) ->
# Pair`), and one puts `left` in the first field, the other `right`. A function given by name is
# not opaque: its own type names it, and `apply` is checked with it
# (`accept_a_function_given_by_name_has_its_own_provenance.em`).

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn apply(f: fn(Span[i32], Span[i32]) -> Pair,
         left: Span[i32], right: Span[i32]) -> Pair:
    return f(left, right) #$ error[E3065]: the returned `Pair` has field provenance that cannot be inferred soundly

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
