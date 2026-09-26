#$ test: compile-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ assert-c: contains("extern int32_t read_one(")

unsafe extern "C":
    @ffi(param(value, borrowed, one))
    safe fn read_one(value: ref i32) -> i32

fn main():
    pass
