#$ test: run-pass
#$ rules: FFI-26, MNG-2
#$ profiles: debug
#$ assert-c: contains("int32_t stable_score(")
#$ stdout: 42

@export("stable_score")
fn score(value: i32) -> i32:
    return value + 1

fn main():
    callback: extern "C" fn(i32) -> i32 = score
    println(callback(41))
