#$ test: compile-fail
#$ rules: FFI-10
#$ error[E5002]: `@ffi(immutable)` cannot describe a `static mut`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(immutable)
    static mut counter: i32

fn main():
    pass
