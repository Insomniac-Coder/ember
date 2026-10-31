#$ test: run-pass
#$ rules: FFI-10, FFI-2
#$ profiles: debug, development, release
#$ stdout: 42

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `abs` requires")
    unsafe fn abs(x: i32) -> i32

fn main():
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        # C's abs has a scalar ABI; this declaration has not asserted safe-to-call.
        println(abs(-42))
