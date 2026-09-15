#$ test: compile-fail
#$ rules: LT-8a, LT-11a, FN-6, FN-6a, TST-20, TST-21
#$ error[E2228]: callable parameter mode mismatch at parameter 0: expected `mut`, found `borrowed`

from std.borrow import with_views2_mut

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    left.push(1)
    right.push(2)
    with_views2_mut(left.as_mut_span(), right.as_mut_span(), fn(a, b) => a[0] + b[0])
