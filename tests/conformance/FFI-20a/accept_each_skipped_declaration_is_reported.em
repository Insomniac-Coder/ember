#$ test: compile-pass
#$ rules: FFI-20a, FFI-38
#$ stdout: 5
#$ warning[W5002]: `Packed` is imported from `unrepresentable.h` only as an opaque type, usable behind a pointer: a union with bit-fields
#$ warning[W5002]: `unrepresentable_hidden` is not imported from `unrepresentable.h`: a global with internal linkage
#$ warning[W5002]: `unrepresentable_log` is not imported from `unrepresentable.h`: variadic (`...`)
#$ warning[W5002]: `unrepresentable_legacy` is not imported from `unrepresentable.h`: declared without a prototype
#$ warning[W5002]: `unrepresentable_number` is not imported from `unrepresentable.h`: parameter `n`: `Packed` by value, which is opaque
#$ help: to use it, declare it by hand in an `unsafe extern "C":` block
# `[FFI-20a]` — every declaration the importer cannot represent is reported with `W5002`, naming
# the construct that defeated it, and a workaround; none is silently absent. `[FFI-38]` — it
# rejects rather than guesses: a union with bit-fields becomes an opaque type, usable only behind a
# pointer (so `unrepresentable_packed_low` is imported), rather than a union whose layout would be
# a guess.

import c "unrepresentable.h"

fn main():
    unsafe:  # SAFETY(ffi): a scalar call
        println(c.unrepresentable_fine(1))
