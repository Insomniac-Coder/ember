#$ test: compile-pass
#$ rules: FFI-11c, FFI-11
#$ profiles: debug

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, borrowed, TODO(count)))
    @safety("callers pass what the C function `inspect` requires")
    fn inspect(data: *u8) -> i32

fn main():
    pass
