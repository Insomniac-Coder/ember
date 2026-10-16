#$ test: compile-fail
#$ rules: UNS-1, FN-6
#$ profiles: debug
#$ help: declare the place `unsafe fn(…)`, and call it inside `unsafe:`
# `[FN-6]` (ODR-115) — an `unsafe fn` coerces only to an `unsafe` callable type; a safe one would let
# a call reach it outside `unsafe`.

unsafe fn increase(x: i32) -> i32:  # SAFETY: this test sets up what the operation requires
    return x + 1

fn main():
    callback: fn(i32) -> i32 = increase   #$ error[E2020]: expected `fn(i32) -> i32`, found `unsafe fn(i32) -> i32`
