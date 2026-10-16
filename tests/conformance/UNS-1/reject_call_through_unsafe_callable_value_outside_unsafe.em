#$ test: compile-fail
#$ rules: UNS-1, FN-6
#$ profiles: debug
# `[UNS-1]`, `[FN-6]` (ODR-115) — a call through a value of an `unsafe` callable type is an `unsafe`
# operation: the value may be any `unsafe fn` of its type.

unsafe fn increase(x: i32) -> i32:  # SAFETY: this test sets up what the operation requires
    return x + 1

fn main():
    callback: unsafe fn(i32) -> i32 = increase
    println(callback(1))   #$ error[E3100]: calling a value of type `unsafe fn(i32) -> i32` requires an `unsafe` block
