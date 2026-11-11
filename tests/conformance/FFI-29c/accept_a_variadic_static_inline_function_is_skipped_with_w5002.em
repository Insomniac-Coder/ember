#$ test: run-pass
#$ rules: FFI-29c
#$ warning[W5002]: `wrap_sum` is not imported from `wrap.h`: a variadic `static inline` function
#$ warning[L3015]: `wrap_twice` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 8
# `[FFI-29c]` — a call to a wrapped `static inline` function costs one extra call; a variadic
# `static inline` function cannot be wrapped and is skipped with `W5002`. `wrap_twice` still
# works; `wrap_sum` is not imported.

import c "wrap.h"

fn main():
    r: i32 = 0
    unsafe:  # SAFETY(ffi): a pure function of a number
        r = c.wrap_twice(4)
    println(r)
