#$ test: compile-fail
#$ rules: LT-35, LT-36, LT-37, LT-41, BRW-1, TST-19

# A named direct callee carries an exact field-access summary, so the sibling
# LT-35 case may mutate `right` before calling `left_value(pair)`. A call
# through `f` is an opaque function-value boundary: the compiler cannot assume
# that its current target is the only callable implementation, so every field
# provenance slot remains required until the call completes.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn left_value(pair: Pair) -> i32:
    return pair.left[0]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(7)
    right: Array[i32] = Array[i32]()
    right.push(8)
    pair = bundle(left.as_span(), right.as_span())
    f: fn(Pair) -> i32 = left_value
    right.push(9) #$ error[E3021]: `right` is borrowed here and mutably borrowed elsewhere
    println(f(pair))
