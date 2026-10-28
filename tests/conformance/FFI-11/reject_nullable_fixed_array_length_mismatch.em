#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `fixed(2)` requires an array of length 2, found 3

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, borrowed, fixed(2), nullable))
    safe fn wrong_length(data: Option[ref [i32; 3]]) -> i32

fn main():
    pass
