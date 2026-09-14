#$ test: run-pass
#$ rules: FN-6b, LT-7, LT-8, LT-10, LT-11, TST-20, TST-21
#$ stdout: 12
#$ stdout: 26

from std.borrow import with_views3_mut, with_views4_mut

fn touch3(mut a: MutSpan[i32], mut b: MutSpan[i32], mut c: MutSpan[i32]) -> i32:
    a[0] = a[0] + 1
    b[0] = b[0] + 2
    c[0] = c[0] + 3
    return a[0] + b[0] + c[0]

fn touch4(mut a: MutSpan[i32], mut b: MutSpan[i32], mut c: MutSpan[i32], mut d: MutSpan[i32]) -> i32:
    a[0] = a[0] + 1
    b[0] = b[0] + 2
    c[0] = c[0] + 3
    d[0] = d[0] + 4
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
    println(with_views3_mut(a.as_mut_span(), b.as_mut_span(), c.as_mut_span(), touch3))
    println(with_views4_mut(a.as_mut_span(), b.as_mut_span(), c.as_mut_span(), d.as_mut_span(), touch4))
