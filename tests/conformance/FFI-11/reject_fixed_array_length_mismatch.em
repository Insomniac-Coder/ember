#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `fixed(4)` requires an array of length 4, found 3

unsafe extern "C":
    @ffi(param(data, borrowed, fixed(4)))
    safe fn wrong_length(data: ref [i32; 3]) -> i32

fn main():
    pass
