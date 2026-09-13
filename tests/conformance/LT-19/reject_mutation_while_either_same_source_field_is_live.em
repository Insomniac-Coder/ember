#$ test: compile-fail
#$ rules: LT-18, LT-19, LT-20, BRW-1, TST-17

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
    values.push(2) #$ error[E3021]: `values` is borrowed here and mutably borrowed elsewhere
    println(pair.right[0])
