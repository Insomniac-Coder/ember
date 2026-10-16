#$ test: compile-fail
#$ rules: UNS-1, FN-6, CLO-3
#$ profiles: debug
# `[UNS-1]`, `[FN-6]` (ODR-115) — a parameter bounded `unsafe fn(…)` may be given an `unsafe fn`, so
# calling it needs `unsafe` whatever this caller passes.

fn apply(f: unsafe fn(i32) -> i32, x: i32) -> i32:
    return f(x)   #$ error[E3100]: calling a value of type `unsafe fn(i32) -> i32` requires an `unsafe` block

fn main():
    println(apply(fn(x) => x, 1))
