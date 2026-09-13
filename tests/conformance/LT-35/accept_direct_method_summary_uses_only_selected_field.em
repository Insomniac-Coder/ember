#$ test: run-pass
#$ rules: LT-35, LT-36, LT-37, TST-19
#$ stdout: 11

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

    fn left_value(self) -> i32:
        return self.left[0]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(11)
    right: Array[i32] = Array[i32]()
    right.push(12)
    pair = Pair(left.as_span(), right.as_span())
    right.push(13)
    println(pair.left_value())
