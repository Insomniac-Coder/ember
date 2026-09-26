#$ test: compile-fail
#$ rules: FFI-11

unsafe extern "C":
    @ffi(result(borrowed, one, from(static)))
    safe fn wrong() -> ref mut i32 #$ error[E5002]: `@ffi` result contract needs a shared reference result

fn main():
    pass
