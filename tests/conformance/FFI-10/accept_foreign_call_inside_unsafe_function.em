#$ test: run-pass
#$ rules: FFI-10, UNS-1
#$ profiles: debug, release, shipping
#$ stdout: 42

unsafe extern "C":
    fn abs(x: i32) -> i32

unsafe fn magnitude(x: i32) -> i32:
    return abs(x)

fn main():
    unsafe:
        println(magnitude(-42))
