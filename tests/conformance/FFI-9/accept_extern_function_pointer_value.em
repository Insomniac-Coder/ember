#$ test: run-pass
#$ rules: FFI-9, FN-6
#$ profiles: debug, release, shipping
#$ stdout: 42

pub extern "C" fn increase(value: i32) -> i32:
    return value + 1

fn main():
    callback: extern "C" fn(i32) -> i32 = increase
    println(callback(41))
