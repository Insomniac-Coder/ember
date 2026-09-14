#$ test: run-pass
#$ rules: FN-6b, LT-7, LT-8, LT-10, TST-20, TST-21
#$ stdout: 3

from std.borrow import with_views2

fn sum(left: Span[i32], right: Span[i32]) -> i32:
    return left[0] + right[0]

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    left.push(1)
    right.push(2)
    println(with_views2(left.as_span(), right.as_span(), sum))
