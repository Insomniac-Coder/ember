#$ test: compile-pass
#$ rules: FFI-11c, FFI-11
#$ profiles: debug

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, TODO(count), from(static)))
    fn inspect() -> *i32

fn main():
    pass
