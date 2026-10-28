#$ test: run-pass
#$ rules: FN-1, BRW-16
#$ stdout: 2
## design rules: R4, R8
## R4 (overlap, exception 1), R8 (f): the same whole variable under two names, proved when compiling,
## is allowed: `add_both(ref mut x, ref mut x)` is compiled against a copy of `add_both` in which `a` and
## `b` are one variable, so both adds reach `x` and `println(a)` prints 2 (verdict 88; Rust refuses it,
## E0499). Other calls of `add_both` use the normal copy, where each `ref mut` parameter is the only way
## in, compiled with noalias.

fn add_both(ref mut a: int, ref mut b: int):
    a += 1
    b += 1
    println(a)

fn main():
    x = 0
    add_both(ref mut x, ref mut x)
