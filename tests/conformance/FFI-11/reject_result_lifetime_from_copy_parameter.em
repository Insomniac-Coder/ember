#$ test: compile-fail
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug
#$ error[E5002]: result lifetime `from(value)` names no borrowed parameter

unsafe extern "C":
    @ffi(result(borrowed, one, from(value)))
    safe fn invalid(value: i32) -> ref i32

fn main():
    pass
