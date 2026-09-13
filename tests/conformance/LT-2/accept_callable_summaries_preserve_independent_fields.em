#$ test: run-pass
#$ rules: LT-2, LT-22, LT-24, LT-35, LT-36, TST-17, TST-19
#$ stdout: 1

# `bundle` returns two independently sourced fields and `first` accesses only
# `a`. The caller may therefore mutate `short` before `first(p)`: neither the
# result summary nor the parameter-access summary may collapse the fields into
# the pre-0.9.5 intersection model. Legacy E3064/B13 is reserved.
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
    short.push(3)
    println(first(p))
