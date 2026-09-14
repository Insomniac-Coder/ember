#$ test: run-pass
#$ rules: LT-35, LT-36, LT-37, LT-41, BRW-1, TST-19
#$ stdout: 7

# An opaque function-value call remains valid when every source field it may
# retain stays available through the call. The rejected companion demonstrates
# that the same conservative all-slot contract prevents a sibling mutation.

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
    println(f(pair))
