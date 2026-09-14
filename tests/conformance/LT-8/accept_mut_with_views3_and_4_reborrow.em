#$ test: run-pass
#$ rules: LT-8, LT-8a, LT-11, FN-2, FN-6a, TST-16, TST-21
#$ stdout: 9
#$ 20

from std.borrow import with_views3_mut, with_views4_mut

fn add3(mut a: MutSpan[i32], mut b: MutSpan[i32], mut c: MutSpan[i32]) -> i32:
    a[0] = a[0] + 1
    b[0] = b[0] + 2
    c[0] = c[0] + 3
    return a[0] + b[0] + c[0]

fn add4(mut a: MutSpan[i32], mut b: MutSpan[i32], mut c: MutSpan[i32],
        mut d: MutSpan[i32]) -> i32:
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
    b.push(1)
    c.push(1)
    d.push(1)
    println(with_views3_mut(a.as_mut_span(), b.as_mut_span(), c.as_mut_span(), add3))
    println(with_views4_mut(a.as_mut_span(), b.as_mut_span(), c.as_mut_span(), d.as_mut_span(), add4))
