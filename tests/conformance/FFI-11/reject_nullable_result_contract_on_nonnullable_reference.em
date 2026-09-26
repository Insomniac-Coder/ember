#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `@ffi` result contract needs a nullable shared reference result

unsafe extern "C":
    @ffi(result(borrowed, one, nullable, from(static)))
    safe fn optional_number() -> ref i32

fn main():
    pass
