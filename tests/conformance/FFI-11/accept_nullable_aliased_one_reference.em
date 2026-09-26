#$ test: compile-pass
#$ rules: FFI-11, FFI-10
#$ profiles: debug
#$ assert-c: contains("typedef int32_t* em_Option_ref_i32;")

unsafe extern "C":
    @ffi(param(value, borrowed, one, aliased, nullable))
    safe fn read_optional(value: Option[ref i32]) -> i32

fn main():
    pass
