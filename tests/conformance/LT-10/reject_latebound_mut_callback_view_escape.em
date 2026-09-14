#$ test: compile-fail
#$ rules: FN-6b, LT-7, LT-8, LT-10, LT-11, TST-20, TST-21
#$ error[E3062]

from std.borrow import with_views2_mut

fn identity(mut left: MutSpan[i32], mut right: MutSpan[i32]) -> MutSpan[i32]:
    return left

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    left.push(1)
    right.push(2)
    escaped: MutSpan[i32] = with_views2_mut(
        left.as_mut_span(), right.as_mut_span(), identity
    )
