#$ test: run-pass
#$ rules: FN-2a
# The call site writes no mode. The callee's declaration makes `value` the
# mutable place borrowed for this call, and the write is visible afterwards.

fn double(mut n: i32):
    n = n + n

fn main():
    value: i32 = 2
    double(value)
    println(value)
#$ stdout: 4
