#$ test: compile-fail
#$ rules: LT-35, LT-36, LT-37, BRW-1, TST-19

@view
struct Pair[T]:
    left: Span[T]
    right: Span[T]

fn right_len[T](pair: Pair[T]) -> usize:
    return pair.right.len()

fn main():
    left: Array[i32] = Array[i32]()
    left.push(41)
    right: Array[i32] = Array[i32]()
    right.push(42)
    pair = Pair[i32](left.as_span(), right.as_span())
    right.push(43) #$ error[E3021]: `right` is borrowed here and mutably borrowed elsewhere
    println(right_len[i32](pair))
