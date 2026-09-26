#$ test: compile-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug

unsafe extern "C":
    @ffi(param(a, borrowed, one), param(b, borrowed, one))
    safe fn compare_one(a: ref i32, b: ref i32) -> i32

fn main():
    pass
