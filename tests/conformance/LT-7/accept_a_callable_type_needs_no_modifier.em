#$ test: run-pass
#$ rules: LT-7, FN-6
#$ stdout: 42
#$ 42
# A callable type needs no modifier: every call through one gets fresh regions
# (`@latebound`, which asked for that, was removed in 0.9.9).

fn apply(f: fn(i32) -> i32, value: i32) -> i32:
    return f(value)

fn answer(value: i32) -> i32:
    return value

fn main():
    println(apply(answer, 42))
    callback: fn(i32) -> i32 = answer
    println(callback(42))
