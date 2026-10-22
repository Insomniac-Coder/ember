#$ test: compile-pass
#$ rules: FFI-11, FFI-10
#$ profiles: debug
#$ assert-c: contains("typedef int32_t* em_3std4core6OptionGaaa9268da143de2d;")
#$ assert-c: contains("extern int32_t em_0x7570646174655f6f7074696f6e616c_15update_optional(")

unsafe extern "C":
    @ffi(param(value, borrowed, one, nullable, exclusive))
    safe fn update_optional(value: Option[ref mut i32]) -> i32

fn main():
    pass
