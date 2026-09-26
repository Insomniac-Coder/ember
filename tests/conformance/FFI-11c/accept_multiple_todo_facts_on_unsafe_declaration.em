#$ test: compile-pass
#$ rules: FFI-11c
#$ profiles: debug

unsafe extern "C":
    @ffi(param(data, TODO(ownership), TODO(count), TODO(nullable), TODO(lifetime)))
    fn inspect(data: *u8) -> i32

fn main():
    pass
