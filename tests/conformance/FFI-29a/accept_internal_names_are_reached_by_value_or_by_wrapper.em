#$ test: run-pass
#$ rules: FFI-29a, FFI-29
#$ warning[L3015]: `internal_bump` is a C function the program calls in unsafe code
#$ warning[W5002]: `internal_counter` is not imported from `internal.h`: a global with internal linkage (`static`)
#$ profiles: debug, development, release
#$ stdout: 9 6
# `[FFI-29a]` — a binding never names a symbol with internal linkage. A `static const` is
# imported as the constant's value; a `static` function is called through the shim's external
# wrapper, which sees the one copy of the header's `static` state.

import c "internal.h"

fn main():
    n: i32 = 0
    unsafe:  # SAFETY(ffi): the function has no preconditions
        n = c.internal_bump()
    println(c.INTERNAL_LIMIT, n)
