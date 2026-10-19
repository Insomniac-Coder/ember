#$ test: compile-fail
#$ rules: FFI-2a
# `[FFI-2a]` — mutability comes from `const` alone: `int*` is a mutable borrow, so a shared view
# does not pass, and `const int*` a shared one, so its result is not a mutable reference.

import c "buffers.h" with (overlay = "buffers_overlay.em")

fn main():
    items: Array[i32] = [4, 5, 6]
    c.buffers_zero(items.as_span())  #$ error[E2020]: expected `MutSpan[i32]`, found `Span[i32]`
    picked: ref mut i32 = c.buffers_pick(items.as_span())  #$ error[E2020]: expected `ref mut i32`, found `ref i32`
