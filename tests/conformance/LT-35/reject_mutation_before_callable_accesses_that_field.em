#$ test: compile-fail
#$ rules: LT-35, LT-36, BRW-1, TST-19

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

fn sum(pair: Pair) -> i32:
    return pair.left[0] + pair.right[0]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(1)
    right: Array[i32] = Array[i32]()
    right.push(2)
    pair = bundle(left.as_span(), right.as_span())
    right.push(3) #$ error[E3021]: `right` is borrowed here and mutably borrowed elsewhere
    println(sum(pair))
