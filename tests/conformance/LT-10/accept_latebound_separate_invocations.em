#$ test: run-pass
#$ rules: FN-6b, LT-7, LT-8, LT-10, LT-11, TST-20, TST-21
#$ stdout: 7
#$ stdout: 10
#$ stdout: 5
#$ stdout: 5

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

    first: i32 = with_views2_mut(left.as_mut_span(), right.as_mut_span(), add)
    second: i32 = with_views2_mut(left.as_mut_span(), right.as_mut_span(), add)
    println(first)
    println(second)
    println(left[0])
    println(right[0])
