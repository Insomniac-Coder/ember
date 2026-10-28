#$ test: compile-fail
#$ rules: FFI-11a
#$ profiles: debug
#$ error[E5012]: pointer contract for `data` has no count

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, borrowed))
    fn inspect(data: *u8, dataLen: usize) -> i32

fn main():
    pass
