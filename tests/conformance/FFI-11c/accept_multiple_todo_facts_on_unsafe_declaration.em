#$ test: compile-pass
#$ rules: FFI-11c
#$ profiles: debug

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, TODO(ownership), TODO(count), TODO(nullable), TODO(lifetime)))
    @safety("callers pass what the C function `inspect` requires")
    fn inspect(data: *u8) -> i32

fn main():
    pass
