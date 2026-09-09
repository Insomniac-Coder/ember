#$ test: run-pass
#$ rules: OWN-7
# "Values of `Copy` types are duplicated bitwise on use; the source remains
# valid." So this is not a move and `x` is still readable — the contrast with
# OWN-1's `Array` is the whole rule.

fn main():
    x: i32 = 1
    y = x
    println(x + y)
#$ stdout: 2
