#$ test: compile-fail
#$ rules: FN-6b, LT-7, LT-8, LT-10, TST-20, TST-21
#$ error[E3062]

from std.borrow import with_views2

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    left.push(1)
    right.push(2)
    escaped: Span[i32] = with_views2(
        left.as_span(), right.as_span(), fn(a: Span[i32], b: Span[i32]) => a
    )
