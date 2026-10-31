#$ test: run-pass
#$ rules: FN-2, FN-6, FN-6a, CLO-1, CLO-3, TST-19
#$ stdout: 5

fn apply(f: fn(ref mut i32) -> i32, ref mut value: i32) -> i32:
    return f(value)

fn main():
    value: i32 = 4
    println(apply(fn(ref mut item) => item + 1, value))
