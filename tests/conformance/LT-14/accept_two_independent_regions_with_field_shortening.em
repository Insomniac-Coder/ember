#$ test: run-pass
#$ rules: LT-14, LT-16, LT-18, LT-19, LT-20, LT-24
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(owns 1)
    right: Array[i32] = Array[i32]()
    right.push(owns 2)
    pair = Pair(left.as_span(), right.as_span())
    println(pair.left[0])
    left.push(owns 3)
    println(pair.right[0])
    println(left[1])
