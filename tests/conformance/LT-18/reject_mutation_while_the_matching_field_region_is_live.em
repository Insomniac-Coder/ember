#$ test: compile-fail
#$ rules: LT-17, LT-18, LT-20, BRW-1, TST-17

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(1)
    right: Array[i32] = Array[i32]()
    right.push(2)
    pair = Pair(left.as_span(), right.as_span())
    left.push(3) #$ error[E3021]: `left` is borrowed here and mutably borrowed elsewhere
    println(pair.left[0])
