#$ test: run-pass
#$ rules: BRW-3
#$ stdout: 2
# `[BRW-3]`'s own example: `v[0]` is copied before the call activates the
# `mut` borrow of `v`, so reading it is allowed.

fn f(mut v: Array[int], x: int):
    v.push(x)

fn main():
    v: Array[int] = [5]
    f(v, v[0])
    println(v.len())
