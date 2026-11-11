#$ test: compile-fail
#$ rules: FFI-29a
#$ warning[W5002]: `internal_counter` is not imported from `internal.h`: a global with internal linkage (`static`)
#$ error[E1010]: `internal_counter` was not imported from `internal.h`: a global with internal linkage (`static`)
#$ profiles: debug
# `[FFI-29a]` — `internal_counter` is a mutable `static`: each C file has its own copy, so no
# binding may name it, and the import has no such name.

import c "internal.h"

fn main():
    println(c.internal_counter)
