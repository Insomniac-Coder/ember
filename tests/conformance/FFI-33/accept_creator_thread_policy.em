#$ test: run-pass
#$ rules: FFI-33, FFI-33c, GRM-37
#$ profiles: debug, release, shipping
#$ assert-c: contains("ember_rt_check_main_thread();")
#$ stdout: 45

@export("creator_score", threads=creator)
pub fn creator_bound() -> i32:
    return 45

fn main():
    println(creator_bound())
