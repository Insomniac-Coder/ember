#$ test: compile-fail
#$ rules: FN-6, CLO-3, UNS-1
#$ profiles: debug
# `[FN-6]` (ODR-115) — a `fn(…)` bound is not met by an `unsafe fn`: the callee calls it outside
# `unsafe`. `unsafe fn(…)` is the bound that takes one.

@safety("this test sets up what the operation requires")
unsafe fn increase(x: i32) -> i32:
    return x + 1

fn apply(f: fn(i32) -> i32, x: i32) -> i32:
    return f(x)

fn main():
    println(apply(increase, 1))   #$ error[E2020]: `apply` cannot take `unsafe fn(i32) -> i32` where it expects `fn(i32) -> i32`
