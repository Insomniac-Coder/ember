#$ test: run-pass
#$ rules: FFI-5a, FFI-29
#$ warning[L3015]: `shape_sum` is a C function the program calls in unsafe code
#$ warning[L3015]: `shape_make` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 13.5
#$ 7 0.5 3
# `[FFI-5a]` — the generated shim holds a `_Static_assert` for the size, alignment and every
# accessed field's offset of each imported aggregate Ember constructs, passes, returns or embeds,
# so the project's own C compiler re-verifies the layout on every build. This program constructs
# `Mixed`, passes it, receives one and reads its fields; it builds only if those assertions hold.

import c "shape.h"

fn main():
    m = c.Mixed(a=7, b=2.5, c=4)
    unsafe:  # SAFETY(ffi): a by-value struct, no pointers
        println(c.shape_sum(m))
    r = m
    unsafe:  # SAFETY(ffi): no pointers
        r = c.shape_make(7)
    println(r.a, r.b, r.c)
