#$ test: compile-fail
#$ rules: FFI-29c
#$ warning[W5002]: `wrap_sum` is not imported from `wrap.h`: a variadic `static inline` function
#$ error[E1010]: `wrap_sum` was not imported from `wrap.h`: a variadic `static inline` function
#$ profiles: debug
# `[FFI-29c]` — the variadic `static inline` function is skipped, so a call to it finds no name.

import c "wrap.h"

fn main():
    unsafe:  # SAFETY(ffi): test
        println(c.wrap_sum(2, 3, 4))
