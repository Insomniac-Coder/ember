#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: result lifetime `from(missing)` names no borrowed parameter

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(value, borrowed, one), result(borrowed, one, from(missing)))
    safe fn inspect(value: ref i32) -> ref i32

fn main():
    pass
