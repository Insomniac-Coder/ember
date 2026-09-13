#$ test: run-pass
#$ rules: LT-20, LT-24, TST-17
#$ stdout: 4
#$ stdout: 5

fn main():
    left: Array[i32] = Array[i32]()
    left.push(4)
    right: Array[i32] = Array[i32]()
    right.push(5)
    pair = (left.as_span(), right.as_span())
    println(pair.0[0])
    left.push(6)
    println(pair.1[0])
    right.push(7)
