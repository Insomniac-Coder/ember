#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `@ffi` names no parameter `missing`

unsafe extern "C":
    @ffi(param(missing, borrowed, one, exclusive))
    fn abs(x: i32) -> i32

fn main():
    pass
