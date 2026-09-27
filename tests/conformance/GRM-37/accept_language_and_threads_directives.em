#! language "0.9.9"
#! threads main
#$ test: run-pass
#$ rules: GRM-37, VER-8, FFI-33c
#$ profiles: debug
#$ assert-c: contains("ember_rt_check_main_thread();")
#$ stdout: 42

@export("combined_score")
fn score(value: i32) -> i32:
    return value + 1

fn main():
    println(score(41))
