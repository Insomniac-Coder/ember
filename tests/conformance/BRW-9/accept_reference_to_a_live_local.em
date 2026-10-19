#$ test: run-pass
#$ rules: BRW-9
#$ profiles: debug, development, release
#$ stdout: 42

fn borrowed_value(value: ref i32) -> i32:
    return value

fn main():
    local: i32 = 42
    println(borrowed_value(ref local))
