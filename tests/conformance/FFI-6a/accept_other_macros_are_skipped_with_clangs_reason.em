#$ test: compile-pass
#$ rules: FFI-6a
#$ stdout: 1
#$ warning[W5001]: macro `REJECTED_TWICE` is not imported from `rejected.h`: a function-like macro
#$ warning[W5001]: macro `REJECTED_TYPE` is not imported from `rejected.h`: Clang: expected expression
#$ warning[W5001]: macro `REJECTED_CALL` is not imported from `rejected.h`: Clang: initializer element is not a compile-time constant
# `[FFI-6a]` — any macro that is not a constant is skipped with `W5001`, giving Clang's reason: a
# function-like macro, a macro that expands to a type, and one whose value is a call.

import c "rejected.h"

fn main():
    println(c.REJECTED_KEPT)
