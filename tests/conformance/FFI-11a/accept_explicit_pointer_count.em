#$ test: compile-pass
#$ rules: FFI-11a
#$ profiles: debug

unsafe extern "C":
    @ffi(param(data, borrowed, one))
    fn inspect(data: *u8) -> i32

fn main():
    pass
