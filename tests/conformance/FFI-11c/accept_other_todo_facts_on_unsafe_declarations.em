#$ test: compile-pass
#$ rules: FFI-11c
#$ profiles: debug

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(a, TODO(ownership), one))
    @safety("callers pass what the C function `inspect_owner` requires")
    unsafe fn inspect_owner(a: *u8) -> i32
    @ffi(param(b, borrowed, one, TODO(nullable)))
    @safety("callers pass what the C function `inspect_nullability` requires")
    unsafe fn inspect_nullability(b: *u8) -> i32
    @ffi(param(c, borrowed, one, TODO(lifetime)))
    @safety("callers pass what the C function `inspect_lifetime` requires")
    unsafe fn inspect_lifetime(c: *u8) -> i32

fn main():
    pass
