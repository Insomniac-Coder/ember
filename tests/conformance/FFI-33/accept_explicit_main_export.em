#$ test: run-pass
#$ rules: FFI-33, FFI-33c, FFI-25
#$ profiles: debug, release
#$ assert-c: contains("ember_rt_thread_attach();")
#$ assert-c: contains("ember_rt_check_main_thread();")
#$ assert-c: contains("int32_t policy_score(")
#$ stdout: 42

@export(threads=main, on_panic=abort, "policy_score")
pub fn score(value: i32) -> i32:
    return value + 1

fn main():
    println(score(41))
