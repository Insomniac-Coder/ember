#$ test: run-pass
#$ rules: LNT-1, CLO-3
#$ stdout: 42

fn twice(x: i32) -> i32:
    return x * 2

fn main():
    operation: fn(i32) -> i32 = twice
    println(operation(21))
