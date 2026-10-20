#$ test: run-pass
#$ rules: FFI-48, FFI-32
#$ profiles: debug
#$ warning[W5002]: `Both` is not imported from `shapes.hpp`
#$ warning[W5002]: `Shared` is not imported from `shapes.hpp`
#$ stdout: 42
# `[FFI-48]` — a header with an unsupported construct still imports the rest: only `Both` and
# `Shared` are refused.

import cpp "shapes.hpp"

fn main():
    unsafe:  # SAFETY(ffi): a scalar call
        println(cpp.answer())
