#$ test: compile-fail
#$ rules: FFI-11

unsafe extern "C":
    @ffi(result(borrowed, fixed(2), exclusive, from(static)))
    safe fn wrong() -> ref [i32; 2] #$ error[E5002]: `@ffi` fixed(2) exclusive result needs a mutable array reference of length 2

fn main():
    pass
