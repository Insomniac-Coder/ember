#$ test: compile-fail
#$ rules: FFI-10, FFI-2
#$ profiles: debug
#$ error[E3100]: `abs(-42)` (line 11) calls the C function `abs`, whose contract is not verified

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `abs` requires")
    fn abs(x: i32) -> i32

fn main():
    println(abs(-42))
