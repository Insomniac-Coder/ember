#$ test: compile-pass
#$ rules: FFI-11, FFI-10
#$ profiles: debug
#$ assert-c: contains("typedef int32_t* em_Option_ref_mut_i32;")
#$ assert-c: contains("extern int32_t update_optional(")

unsafe extern "C":
    @ffi(param(value, borrowed, one, nullable, exclusive))
    safe fn update_optional(value: Option[ref mut i32]) -> i32

fn main():
    pass
