#$ test: compile-pass
#$ rules: FFI-11, FFI-10
#$ profiles: debug
#$ assert-c: contains("typedef int32_t* em_3std4core6OptionGacdf02bf7ba2defe;")

unsafe extern "C":
    @ffi(param(value, borrowed, one, aliased, nullable))
    safe fn read_optional(value: Option[ref i32]) -> i32

fn main():
    pass
