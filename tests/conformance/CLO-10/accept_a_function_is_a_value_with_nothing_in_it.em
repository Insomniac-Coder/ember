#$ test: run-pass
#$ rules: CLO-10, CLO-3, CLO-1, FN-6
#$ profiles: debug, development, release
#$ stdout: 11 14 true false
#$ assert-c: contains("ember_fn_value")

# `[CLO-3]`, ODR-102 — a function, or a lambda that captures nothing, is a
# value the program may copy; it becomes an owned callable value with no
# state where one is wanted, and two compare equal when they call the same
# function with the same state.

fn inc(x: int) -> int:
    return x + 1

fn relay(g: fn(int) -> int) -> int:
    return (0..2).map(g).sum()

fn main():
    f: fn(int) -> int = inc
    same: fn(int) -> int = inc
    other: fn(int) -> int = fn(x) => x * 2
    println(f(10), relay(inc) + 11, f == same, f == other)
