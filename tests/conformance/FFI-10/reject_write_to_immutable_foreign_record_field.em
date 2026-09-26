#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E2140]: a foreign static without `mut` cannot be assigned

@derive(Copy)
struct Pair:
    left: i32

unsafe extern "C":
    @ffi(immutable)
    static frozen_pair: Pair

fn main():
    unsafe:
        frozen_pair.left = 12
