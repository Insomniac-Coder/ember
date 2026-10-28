#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `fixed(2)` with `exclusive` requires a mutable array reference

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, borrowed, fixed(2), exclusive))
    safe fn wrong_mutability(data: ref [i32; 2]) -> i32

fn main():
    pass
