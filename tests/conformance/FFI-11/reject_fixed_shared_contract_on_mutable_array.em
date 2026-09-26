#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `fixed(2)` without `exclusive` requires a shared array reference

unsafe extern "C":
    @ffi(param(data, borrowed, fixed(2)))
    safe fn wrong_mutability(data: ref mut [i32; 2]) -> i32

fn main():
    pass
