#$ test: run-pass
#$ rules: FFI-10, FFI-9
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains("extern int32_t abs(")

unsafe extern "C":
    safe fn abs(x: i32) -> i32

fn main():
    println(abs(-42))
