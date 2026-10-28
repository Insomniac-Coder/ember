#$ test: compile-fail
#$ rules: FFI-11

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, one, from(static)))
    safe fn wrong() -> ref mut i32 #$ error[E5002]: `@ffi` result contract needs a shared reference result

fn main():
    pass
