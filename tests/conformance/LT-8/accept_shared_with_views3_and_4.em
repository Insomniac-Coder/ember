#$ test: run-pass
#$ rules: LT-8, LT-8a, LT-9, FN-6a, TST-16, TST-21
#$ stdout: 6
#$ 10

from std.borrow import with_views3, with_views4

fn sum3(a: Span[i32], b: Span[i32], c: Span[i32]) -> i32:
    return a[0] + b[0] + c[0]

fn sum4(a: Span[i32], b: Span[i32], c: Span[i32], d: Span[i32]) -> i32:
    return a[0] + b[0] + c[0] + d[0]

fn main():
    a: Array[i32] = Array[i32]()
    b: Array[i32] = Array[i32]()
    c: Array[i32] = Array[i32]()
    d: Array[i32] = Array[i32]()
    a.push(1)
    b.push(2)
    c.push(3)
    d.push(4)
    println(with_views3(a.as_span(), b.as_span(), c.as_span(), sum3))
    println(with_views4(a.as_span(), b.as_span(), c.as_span(), d.as_span(), sum4))
