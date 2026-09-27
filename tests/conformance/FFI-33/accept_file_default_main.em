#! threads main
#$ test: run-pass
#$ rules: FFI-33, FFI-33c, GRM-37
#$ profiles: debug, release
#$ assert-c: contains("ember_rt_thread_attach();")
#$ assert-c: contains("ember_rt_check_main_thread();")
#$ stdout: 42

pub extern "C" fn plain_score(value: i32) -> i32:
    return value + 1

fn main():
    println(plain_score(41))
