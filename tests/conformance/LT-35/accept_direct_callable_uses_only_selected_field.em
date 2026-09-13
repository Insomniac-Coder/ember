#$ test: run-pass
#$ rules: LT-24, LT-35, LT-36, TST-17, TST-19
#$ stdout: 7

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
    right.push(9)
    println(left_value(pair))
