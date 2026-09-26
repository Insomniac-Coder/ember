#$ test: compile-fail
#$ rules: FN-6, FFI-9
#$ profiles: debug
#$ error[E2020]: expected `extern "C" fn(i32) -> i32`, found `closure3_env`

fn main():
    offset: i32 = 1
    callback: extern "C" fn(i32) -> i32 = fn(value: i32) -> i32 => value + offset
    println(callback(41))
