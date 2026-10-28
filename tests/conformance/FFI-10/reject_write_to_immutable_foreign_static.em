#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E2140]: a foreign static without `mut` cannot be assigned

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(immutable)
    static frozen_count: i32

fn main():
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        frozen_count = 9
