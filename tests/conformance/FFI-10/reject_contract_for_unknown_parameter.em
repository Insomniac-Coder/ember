#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `@ffi` names no parameter `missing`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(missing, borrowed, one, exclusive))
    @safety("callers pass what the C function `abs` requires")
    fn abs(x: i32) -> i32

fn main():
    pass
