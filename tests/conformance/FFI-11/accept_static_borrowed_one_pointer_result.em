#$ test: compile-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ assert-c: contains("extern int32_t* static_number(")

unsafe extern "C":
    @ffi(result(borrowed, one, from(static)))
    safe fn static_number() -> ref i32

fn main():
    pass
