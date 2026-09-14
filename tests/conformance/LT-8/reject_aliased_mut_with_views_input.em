#$ test: compile-fail
#$ rules: LT-8, LT-11, BRW-4, TST-16, TST-21
#$ error[E3022]: `values` is already mutably borrowed

from std.borrow import with_views2_mut

fn touch(mut a: MutSpan[i32], mut b: MutSpan[i32]):
    a[0] = b[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    with_views2_mut(values.as_mut_span(), values.as_mut_span(), touch)
