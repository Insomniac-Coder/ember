#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `@ffi` fixed(2) result needs a shared array reference of length 2

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, fixed(2), from(static)))
    safe fn pair() -> ref [i32; 3]

fn main():
    pass
