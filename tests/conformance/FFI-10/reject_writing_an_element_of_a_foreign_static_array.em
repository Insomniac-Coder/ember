#$ test: compile-fail
#$ rules: FFI-10, STA-1
# `[FFI-10]` — an element of an array inside a foreign static is C storage,
# written in place inside `unsafe` when the static is `static mut` (D1 wrote a
# copy). A static without `mut` is not written, and a write outside `unsafe`
# is not allowed either.

@derive(Copy)
struct Table:
    values: [i32; 2]

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static mut table: Table
    static mut samples: [i32; 4]
    @ffi(immutable)
    static frozen: Table

fn main():
    unsafe:  # SAFETY(ffi): each global is declared as C defines it
        table.values[0] = 99
        samples[1] += 2
    frozen.values[0] = 5    #$ error[E2140]: a foreign static without `mut` cannot be assigned
    samples[2] = 1    #$ error[E3100]: writing a foreign static requires `unsafe`
