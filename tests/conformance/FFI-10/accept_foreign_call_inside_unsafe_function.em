#$ test: run-pass
#$ rules: FFI-10, UNS-1
#$ profiles: debug, development, release
#$ stdout: 42

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `abs` requires")
    unsafe fn abs(x: i32) -> i32

@safety("each foreign call matches its C declaration")
unsafe fn magnitude(x: i32) -> i32:
    return abs(x)

fn main():
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        println(magnitude(-42))
