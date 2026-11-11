#$ test: run-pass
#$ rules: FFI-29, FFI-29c
#$ warning[L3015]: `inline_twice` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 42
# `[FFI-29]` — the shim holds an external wrapper for every `static inline` function Ember calls:
# `inline_twice` has no symbol of its own, and the call below links through the wrapper.

import c "inline.c"

fn main():
    r: i32 = 0
    unsafe:  # SAFETY(ffi): a pure function of a number
        r = c.inline_twice(21)
    println(r)
