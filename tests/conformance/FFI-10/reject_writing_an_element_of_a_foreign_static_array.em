#$ test: compile-fail
#$ rules: FFI-10
#$ error[E0900]: writing an element of an array inside a foreign static is not implemented yet
#$ error[E2140]: a foreign static without `mut` cannot be assigned
# `[FFI-10]` — an element of an array inside a foreign static is C storage. Writing
# one wrote a copy of the static, so the write was lost, and on a static without
# `mut` it was accepted (D1). Until it is written in place it is refused, and the
# immutable static is refused as any write to it is.

@derive(Copy)
struct Table:
    values: [i32; 2]

unsafe extern "C":
    static mut table: Table
    @ffi(immutable)
    static frozen: Table

fn main():
    unsafe:
        table.values[0] = 99
    frozen.values[0] = 5
