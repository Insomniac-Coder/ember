#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `@ffi` contract for `x` needs a pointer carrier

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(x, borrowed, one, exclusive))
    @safety("callers pass what the C function `abs` requires")
    unsafe fn abs(x: i32) -> i32

fn main():
    pass
