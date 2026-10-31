#$ test: run-pass
#$ rules: LT-35, LT-36, LT-37, TST-17, TST-19
#$ stdout: 1

@view
[T]
struct Pair:
    left: Span[T]
    right: Span[T]

[T]
fn left_len(pair: Pair[T]) -> int:
    return pair.left.len()

fn main():
    left: Array[i32] = Array[i32]()
    left.push(11)
    right: Array[i32] = Array[i32]()
    right.push(12)
    pair = Pair[i32](left.as_span(), right.as_span())
    right.push(13)
    println(left_len[i32](pair))
