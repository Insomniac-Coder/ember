#$ test: compile-fail
#$ rules: LT-5, BRW-1
# `[LT-5]` — diagnostics describe regions in source terms: the view of `xs` is still used on a
# later line.

fn main():
    xs: Array[i32] = Array[i32]()
    xs.push(owns 1)
    v = ref xs[0]
    xs.push(owns 2)                 #$ error[E3021]: `xs[0]` is borrowed here
    println(v)
