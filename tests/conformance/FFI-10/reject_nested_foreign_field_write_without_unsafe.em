#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E5002]: writing a foreign static requires `unsafe`

@derive(Copy)
struct Inner:
    value: i32

@derive(Copy)
struct Outer:
    inner: Inner

unsafe extern "C":
    static mut foreign_outer: Outer

fn main():
    foreign_outer.inner.value = 12
