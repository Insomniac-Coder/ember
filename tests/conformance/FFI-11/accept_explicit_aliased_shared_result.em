#$ test: compile-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug

unsafe extern "C":
    @ffi(result(borrowed, one, aliased, from(static)))
    safe fn shared() -> ref i32

fn main():
    pass
