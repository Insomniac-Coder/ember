#$ test: compile-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ assert-c: contains("extern int32_t* em_0x7374617469635f6e756d626572_13static_number(")

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, one, from(static)))
    safe fn static_number() -> ref i32

fn main():
    pass
