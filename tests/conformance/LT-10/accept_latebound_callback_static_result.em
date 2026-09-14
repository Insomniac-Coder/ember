#$ test: compile-pass
#$ rules: FN-6b, LT-7, LT-8, LT-10, TST-20, TST-21

from std.borrow import with_views2

fn static_result(left: Span[i32], right: Span[i32]) -> str:
    return "ok"

fn main():
    left: Array[i32] = Array[i32]()
    right: Array[i32] = Array[i32]()
    left.push(1)
    right.push(2)
    stored: Box[str] = Box(with_views2(left.as_span(), right.as_span(), static_result))
    println(stored.get())
