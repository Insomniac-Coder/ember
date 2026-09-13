#$ test: run-pass
#$ rules: LT-21, LT-22, LT-35
#$ stdout: 2

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn identity(value: Span[i32]) -> Span[i32]:
    return value

fn main():
    left: Array[i32] = Array[i32]()
    left.push(1)
    right: Array[i32] = Array[i32]()
    right.push(2)
    pair = Pair(left.as_span(), right.as_span())
    pair.left = identity(right.as_span())
    println(pair.left[0])
