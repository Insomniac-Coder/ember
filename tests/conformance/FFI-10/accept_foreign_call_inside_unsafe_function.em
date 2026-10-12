#$ test: run-pass
#$ rules: FFI-10, UNS-1
#$ profiles: debug, release, shipping
#$ stdout: 42

unsafe extern "C":
    fn abs(x: i32) -> i32

unsafe fn magnitude(x: i32) -> i32:  # SAFETY(ffi): each foreign call matches its C declaration
    return abs(x)

fn main():
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        println(magnitude(-42))
