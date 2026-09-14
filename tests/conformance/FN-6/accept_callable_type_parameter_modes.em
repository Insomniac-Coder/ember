#$ test: run-pass
#$ rules: FN-2, FN-6, FN-6a, CLO-3, TST-19
#$ stdout: 5
#$ 6

# Callable types carry parameter modes.  `mut` remains a mutable borrow at
# both the indirect-call boundary and the eventual named-function boundary.

fn increment(mut value: i32) -> i32:
    value = value + 1
    return value

fn apply(f: fn(mut i32) -> i32, mut value: i32) -> i32:
    return f(value)

fn main():
    value = 4
    operation: fn(mut i32) -> i32 = increment
    println(operation(value))
    println(apply(increment, value))
