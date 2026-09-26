#$ test: compile-pass
#$ rules: FFI-11c, FFI-11
#$ profiles: debug

unsafe extern "C":
    @ffi(result(borrowed, TODO(count), from(static)))
    fn inspect() -> *i32

fn main():
    pass
