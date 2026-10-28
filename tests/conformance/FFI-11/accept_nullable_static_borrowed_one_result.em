#$ test: compile-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ assert-c: contains("typedef int32_t* em_3std4core6OptionGacdf02bf7ba2defe;")
#$ assert-c: contains("extern em_3std4core6OptionGacdf02bf7ba2defe em_0x6f7074696f6e616c5f6e756d626572_15optional_number(")

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, one, nullable, from(static)))
    safe fn optional_number() -> Option[ref i32]

fn main():
    pass
