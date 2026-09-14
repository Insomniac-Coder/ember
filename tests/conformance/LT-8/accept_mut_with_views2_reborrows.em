#$ test: run-pass
#$ rules: LT-8, LT-8a, LT-11, FN-2, FN-6a, TST-16, TST-21
#$ stdout: 7
#$ 4
#$ 3

from std.borrow import with_views2_mut

fn add(mut left: MutSpan[i32], mut right: MutSpan[i32]) -> i32:
    left[0] = left[0] + 1
    right[0] = right[0] + 2
    return left[0] + right[0]

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    left.push(3)
    right.push(1)
    println(with_views2_mut(left.as_mut_span(), right.as_mut_span(), add))
    println(left[0])
    println(right[0])
