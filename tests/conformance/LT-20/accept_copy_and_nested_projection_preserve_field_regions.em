#$ test: run-pass
#$ rules: LT-20, LT-24, LT-31a, TST-17
#$ stdout: 10
#$ stdout: 20
#$ stdout: 30

@view
@derive(Copy)
struct Pair:
    left: Span[i32]
    right: Span[i32]

@view
@derive(Copy)
struct Nested:
    pair: Pair
    tag: i32

fn main():
    left: Array[i32] = Array[i32]()
    left.push(10)
    right: Array[i32] = Array[i32]()
    right.push(20)
    original = Nested(Pair(left.as_span(), right.as_span()), 30)
    copied = original
    println(copied.pair.left[0])
    left.push(11)
    println(copied.pair.right[0])
    right.push(21)
    println(copied.tag)
