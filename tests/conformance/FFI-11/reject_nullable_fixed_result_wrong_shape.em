#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `@ffi` nullable fixed(2) result needs an optional shared array reference of length 2

unsafe extern "C":
    @ffi(result(borrowed, fixed(2), nullable, from(static)))
    safe fn pair() -> Option[ref i32]

fn main():
    pass
