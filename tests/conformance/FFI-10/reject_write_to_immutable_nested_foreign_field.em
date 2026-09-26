#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E2140]: a foreign static without `mut` cannot be assigned

@derive(Copy)
struct Inner:
    value: i32

@derive(Copy)
struct Outer:
    inner: Inner

unsafe extern "C":
    @ffi(immutable)
    static foreign_outer: Outer

fn main():
    unsafe:
        foreign_outer.inner.value = 12
