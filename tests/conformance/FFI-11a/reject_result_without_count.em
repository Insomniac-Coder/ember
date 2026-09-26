#$ test: compile-fail
#$ rules: FFI-11a
#$ profiles: debug
#$ error[E5012]: pointer result contract has no count

unsafe extern "C":
    @ffi(result(borrowed, from(static)))
    fn inspect(dataLen: usize) -> *i32

fn main():
    pass
