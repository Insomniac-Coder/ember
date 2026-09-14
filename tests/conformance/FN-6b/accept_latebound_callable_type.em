#$ test: run-pass
#$ rules: FN-6b, FN-6a, TST-20, TST-21
#$ stdout: 42
#$ 42

fn apply(f: @latebound fn(i32) -> i32, value: i32) -> i32:
    return f(value)

fn answer(value: i32) -> i32:
    return value

fn main():
    println(apply(answer, 42))
    callback: @latebound fn(i32) -> i32 = answer
    println(callback(42))
