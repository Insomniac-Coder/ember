#! threads main
#$ test: run-pass
#$ rules: FFI-33, GRM-37
#$ profiles: debug, release
#$ assert-c: !contains("ember_rt_check_main_thread();")
#$ stdout: 42

@export(threads=any)
pub fn score(value: i32) -> i32:
    return value + 1

fn main():
    println(score(41))
