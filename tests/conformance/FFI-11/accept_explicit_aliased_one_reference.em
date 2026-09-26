#$ test: compile-pass
#$ rules: FFI-11, FFI-10
#$ profiles: debug
#$ assert-c: contains("extern int32_t read_one(int32_t*")

unsafe extern "C":
    @ffi(param(value, borrowed, one, aliased))
    safe fn read_one(value: ref i32) -> i32

fn main():
    pass
