#$ test: compile-fail
#$ rules: LT-35, LT-36, LT-37, BRW-1, TST-19

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

    fn right_value(self) -> i32:
        return self.right[0]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(31)
    right: Array[i32] = Array[i32]()
    right.push(32)
    pair = Pair(left.as_span(), right.as_span())
    right.push(33) #$ error[E3021]: `right` is borrowed here and mutably borrowed elsewhere
    println(pair.right_value())
