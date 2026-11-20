#$ test: compile-fail
#$ rules: LT-5, BRW-1
# `[LT-5]` — diagnostics describe regions in source terms: the view of `xs` is still used on a
# later line. (A push would not end it: `v` is a position, `[BRW-16]`; a clear does.)

fn main():
    xs: Array[i32] = Array[i32]()
    xs.push(owns 1)
    v = ref xs[0]
    xs.clear()
    println(v)    #$ error[E3060]
