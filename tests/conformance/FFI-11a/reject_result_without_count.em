#$ test: compile-fail
#$ rules: FFI-11a
#$ profiles: debug
#$ error[E5012]: pointer result contract has no count

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, from(static)))
    @safety("callers pass what the C function `inspect` requires")
    unsafe fn inspect(dataLen: usize) -> *i32

fn main():
    pass
