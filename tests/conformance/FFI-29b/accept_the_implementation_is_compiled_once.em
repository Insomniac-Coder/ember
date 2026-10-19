#$ test: run-pass
#$ rules: FFI-29b
#$ profiles: debug, development, release
#$ stdout: 42 42 2
# `[FFI-29b]` — `implementation = ["SINGLE_IMPLEMENTATION"]` compiles a single-header library's
# implementation (into the import's shim, with the macro defined); the same import written twice
# is one module, so the implementation is compiled exactly once and links.

import c "single.h" with (implementation = ["SINGLE_IMPLEMENTATION"])
import c "single.h" with (implementation = ["SINGLE_IMPLEMENTATION"]) as again

fn main():
    unsafe:  # SAFETY(ffi): scalar calls
        println(c.single_answer(), again.single_answer(), c.single_calls())
