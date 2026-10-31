#$ test: compile-pass
#$ rules: FFI-11a
#$ profiles: debug

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, borrowed, one))
    @safety("callers pass what the C function `inspect` requires")
    unsafe fn inspect(data: *u8) -> i32

fn main():
    pass
