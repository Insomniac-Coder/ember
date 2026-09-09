#$ test: run-pass
#$ rules: BRW-3, FN-1
# The second half of `[BRW-3]`'s scope sentence: "`mut` arguments whose argument
# expression is a simple place". `a` is reserved for the `mut` parameter while
# `a.len()` evaluates in between, and the call compiles. `a` stays the
# caller's afterwards: `[FN-1]` takes a borrow, not a move.

fn set_first(mut a: Array[i32], v: i32):
    a.push(v)

fn main():
    a: Array[i32] = Array[i32]()
    a.push(7)
    set_first(a, a.len() as i32)
    println(a[0])
    println(a[1])
#$ stdout: 7
#$ 1
