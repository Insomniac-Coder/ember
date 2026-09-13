#$ test: run-pass
#$ rules: LT-18, LT-19, LT-20, BRW-2, TST-17
#$ stdout: 1
#$ stdout: 1
#$ stdout: 2

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    shared = values.as_span()
    pair = Pair(shared, shared)
    println(pair.left[0])
    println(pair.right[0])
    values.push(2)
    println(values[1])
