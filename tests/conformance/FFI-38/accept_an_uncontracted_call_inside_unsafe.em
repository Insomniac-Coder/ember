#$ test: run-pass
#$ rules: FFI-38
#$ warning[L3015]: `uncontracted_add` is a C function the program calls in unsafe code
#$ warning[L3015]: `uncontracted_sum` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 3 6
# `[FFI-38]` — an uncontracted import is not refused: its calls are the caller's assertion, inside
# `unsafe`, and the raw pointer the header declares is what the call takes.

import c "uncontracted.h"

fn main():
    xs: [i32; 3] = [1, 2, 3]
    unsafe:  # SAFETY(ffi): `items` points at three live `i32`s and `count` says three
        println(c.uncontracted_add(1, 2), c.uncontracted_sum(ref_to_ptr(ref xs[0]), 3))
