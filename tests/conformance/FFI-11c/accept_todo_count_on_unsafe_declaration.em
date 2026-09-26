#$ test: compile-pass
#$ rules: FFI-11c, FFI-11
#$ profiles: debug

unsafe extern "C":
    @ffi(param(data, borrowed, TODO(count)))
    fn inspect(data: *u8) -> i32

fn main():
    pass
