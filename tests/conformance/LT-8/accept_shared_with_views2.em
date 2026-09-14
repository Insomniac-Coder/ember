#$ test: run-pass
#$ rules: LT-8, LT-8a, LT-9, FN-6, FN-6a, TST-16, TST-21
#$ stdout: 7

from std.borrow import with_views2

fn sum(left: Span[i32], right: Span[i32]) -> i32:
    return left[0] + right[0]

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    left.push(3)
    right.push(4)
    println(with_views2(left.as_span(), right.as_span(), sum))
