#$ test: run-pass
#$ rules: LT-22, LT-24, LT-35, LT-40, TST-17, TST-19
#$ stdout: 22

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

# Declared before its dependency deliberately: summaries are a call-graph
# fixpoint, not a source-order accident.
fn outer(left: Span[i32], right: Span[i32]) -> Pair:
    return inner(left, right)

fn inner(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn right_value(pair: Pair) -> i32:
    return pair.right[0]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(11)
    right: Array[i32] = Array[i32]()
    right.push(22)
    pair = outer(left.as_span(), right.as_span())
    left.push(33)
    println(right_value(pair))
