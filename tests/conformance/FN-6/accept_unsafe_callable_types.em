#$ test: run-pass
#$ rules: FN-6, UNS-1, CLO-3, CLO-6
# `[FN-6]` (ODR-115) — `unsafe fn(…) -> R` holds an `unsafe fn` or any function or lambda, and each
# call through it is inside `unsafe:`; as a bound, `unsafe fn(…)` takes all three; it combines with
# `once` and with `extern "C"` (`once unsafe extern "C" fn`).

unsafe fn increase(x: i32) -> i32:  # SAFETY: this test sets up what the operation requires
    return x + 1

fn double(x: i32) -> i32:
    return x * 2

fn apply(f: unsafe fn(i32) -> i32, x: i32) -> i32:
    unsafe:  # SAFETY: every caller passes a function with no obligation
        return f(x)

fn run_once(f: once unsafe fn(i32) -> i32) -> i32:
    unsafe:  # SAFETY: every caller passes a function with no obligation
        return f(1)

fn main():
    callback: unsafe fn(i32) -> i32 = increase
    other: unsafe fn(i32) -> i32 = double
    pointer: unsafe extern "C" fn(i32) -> i32 = double
    unsafe:  # SAFETY: neither function has an obligation
        println(callback(1) + other(2))
        println(pointer(5))
    println(apply(increase, 3) + apply(double, 4) + apply(fn(x) => x - 1, 5))
    total: i32 = 10
    println(run_once(double) + run_once(fn(x) => x + total))
#$ stdout: 6
#$ 10
#$ 16
#$ 13
