#$ test: run-pass
#$ rules: FFI-10, FFI-9, FN-6
#$ profiles: debug, development, release
#$ stdout: 42

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    safe fn abs(x: i32) -> i32

fn main():
    callback: extern "C" fn(i32) -> i32 = abs
    println(callback(-42))
