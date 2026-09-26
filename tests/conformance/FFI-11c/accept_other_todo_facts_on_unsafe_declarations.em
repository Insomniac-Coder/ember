#$ test: compile-pass
#$ rules: FFI-11c
#$ profiles: debug

unsafe extern "C":
    @ffi(param(a, TODO(ownership), one))
    fn inspect_owner(a: *u8) -> i32
    @ffi(param(b, borrowed, one, TODO(nullable)))
    fn inspect_nullability(b: *u8) -> i32
    @ffi(param(c, borrowed, one, TODO(lifetime)))
    fn inspect_lifetime(c: *u8) -> i32

fn main():
    pass
