#$ test: compile-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ assert-c: contains("typedef int32_t* em_Option_ref_i32;")
#$ assert-c: contains("extern em_Option_ref_i32 optional_number(")

unsafe extern "C":
    @ffi(result(borrowed, one, nullable, from(static)))
    safe fn optional_number() -> Option[ref i32]

fn main():
    pass
