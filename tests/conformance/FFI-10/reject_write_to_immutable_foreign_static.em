#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E2140]: a foreign static without `mut` cannot be assigned

unsafe extern "C":
    @ffi(immutable)
    static frozen_count: i32

fn main():
    unsafe:
        frozen_count = 9
