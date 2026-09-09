#$ test: compile-fail
#$ rules: LT-2, DIA-7a
# `[LT-2]`: "Constructing a view struct from several references gives it the
# intersection of their regions… Multiple independent regions inside one
# struct are not expressible in v1; nest structs or copy data."
#
# So `p` holds **both** loans for as long as any part of it is live, and
# touching `p` at all keeps `short`'s loan alive — even though the only field
# ever read is `a`, which points into `long`. That is `[DIA-7a]`'s shape B13,
# "a view struct needing two independent regions", and its code is `E3064`.
#
# D-011 recorded `E3064` as "registered and emitted by nothing". It was
# reachable the whole time; this program was being reported as B3, whose help
# — shorten the borrow's last use — is no fix here, because the use keeping
# the loan alive is of the *other* field.

@view
struct Pair:
    a: Span[i32]
    b: Span[i32]

fn bundle(x: Span[i32], y: Span[i32]) -> Pair:
    return Pair(x, y)

fn first(p: Pair) -> i32:
    return p.a[0]

fn main():
    long: Array[i32] = Array[i32]()
    long.push(1)
    short: Array[i32] = Array[i32]()
    short.push(2)
    p = bundle(long, short)
    short.push(3)          #$ error[E3064]: `short` is borrowed through a view struct that bundles two views
    println(first(p))
