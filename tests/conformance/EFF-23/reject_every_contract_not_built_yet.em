#$ test: compile-fail
#$ rules: EFF-23, PHIL-12
#$ profiles: debug
#$ error[E0900]: `@noalloc` is not implemented yet
#$ error[E0900]: `@nosync` is not implemented yet
#$ error[E0900]: `@noblock` is not implemented yet
#$ error[E0900]: `@noio` is not implemented yet
#$ error[E0900]: `@nolock` is not implemented yet
#$ error[E0900]: `@nopanic` is not implemented yet
#$ error[E0900]: `@realtime` is not implemented yet
#$ error[E0900]: `@deterministic` is not implemented yet
# `[EFF-23]` — a contract attribute whose checking is not built is rejected with `E0900`, never
# accepted unchecked. (`@static_safe` is built, `[EFF-12]`.) Each case leaves this file as its
# contract is built.

@noalloc
fn a() -> int:
    return 1

@nosync
fn b() -> int:
    return 2

@noblock
fn c() -> int:
    return 3

@noio
fn d() -> int:
    return 4

@nolock
fn e() -> int:
    return 5

@nopanic(explicit)
fn f() -> int:
    return 6

@realtime
fn g() -> int:
    return 7

@deterministic
fn h() -> int:
    return 8

fn main():
    println(a() + b() + c() + d() + e() + f() + g() + h())
