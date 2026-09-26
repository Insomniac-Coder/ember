#$ test: compile-fail
#$ rules: FFI-10
#$ error[E5002]: `@ffi(immutable)` cannot describe a `static mut`

unsafe extern "C":
    @ffi(immutable)
    static mut counter: i32

fn main():
    pass
