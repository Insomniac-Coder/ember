#$ test: compile-fail
#$ rules: FFI-11

unsafe extern "C":
    @ffi(result(borrowed, one, exclusive, from(static)))
    safe fn wrong() -> ref i32 #$ error[E5002]: `@ffi` exclusive result contract needs a mutable reference result

fn main():
    pass
