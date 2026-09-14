#$ test: run-pass
#$ rules: FN-6b, LT-7, LT-8, LT-10, LT-3, TYP-15, HEAP-1, DRP-6, TST-20, TST-21
#$ stdout: stable

from std.borrow import with_views2

static GREETING: str = "stable"

fn static_result(left: Span[i32], right: Span[i32]) -> str:
    return GREETING

fn main():
    left: Array[i32] = Array[i32]()
    left.push(1)
    right: Array[i32] = Array[i32]()
    right.push(2)
    stored: Box[str] = Box(with_views2(left.as_span(), right.as_span(), static_result))
    println(stored.get())
