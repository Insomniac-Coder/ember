#$ test: run-pass
#$ rules: FN-6b, LT-7, LT-8, LT-10, TST-20, TST-21
#$ stdout: 1

from std.borrow import with_views2

fn first(view: Span[i32]) -> i32:
    return view[0]

fn apply_one(f: @latebound fn(Span[i32]) -> i32, view: Span[i32]) -> i32:
    return f(view)

fn nested(left: Span[i32], right: Span[i32]) -> i32:
    return apply_one(first, left)

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    left.push(1)
    right.push(2)
    println(with_views2(left.as_span(), right.as_span(), nested))
