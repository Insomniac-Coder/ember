#$ test: compile-fail
#$ rules: FFI-10, FFI-2
#$ profiles: debug
#$ error[E5002]: a foreign call without a complete safe contract requires `unsafe`

unsafe extern "C":
    fn abs(x: i32) -> i32

fn main():
    println(abs(-42))
