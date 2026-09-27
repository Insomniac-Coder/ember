#! threads creator
#$ test: run-pass
#$ rules: FFI-33, FFI-33c, GRM-37
#$ profiles: debug, release
#$ assert-c: contains("ember_rt_check_main_thread();")
#$ stdout: 43 44

@export("default_creator_score")
pub fn default_creator() -> i32:
    return 43

@export("override_any_score", threads=any)
pub fn override_any() -> i32:
    return 44

fn main():
    println(default_creator(), override_any())
