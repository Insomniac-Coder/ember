#$ test: compile-pass
#$ rules: FFI-20a, FFI-38
#$ stdout: 5
#$ warning[W5002]: `Number` is imported from `unrepresentable.h` only as an opaque type, usable behind a pointer: a C union
#$ warning[W5002]: `Flags` is imported from `unrepresentable.h` only as an opaque type, usable behind a pointer: bit-field `mode` of type `enum Mode`
#$ warning[W5002]: `unrepresentable_hidden` is not imported from `unrepresentable.h`: a global with internal linkage
#$ warning[W5002]: `unrepresentable_log` is not imported from `unrepresentable.h`: variadic (`...`)
#$ warning[W5002]: `unrepresentable_precise` is not imported from `unrepresentable.h`: its result: `long double`
#$ warning[W5002]: `unrepresentable_number` is not imported from `unrepresentable.h`: parameter `n`: `Number` by value, which is opaque
#$ help: to use it, declare it by hand in an `unsafe extern "C":` block
# `[FFI-20a]` — every declaration the importer cannot represent is reported with `W5002`, naming
# the construct that defeated it, and a workaround; none is silently absent. `[FFI-38]` — it
# rejects rather than guesses: a union and a struct with an enum bit-field (only integer and `bool`
# bit-fields have accessors yet) become opaque types, usable only behind a pointer (so
# `unrepresentable_flags_ready` is imported), rather than structs whose layout would be a guess.

import c "unrepresentable.h"

fn main():
    unsafe:  # SAFETY(ffi): a scalar call
        println(c.unrepresentable_fine(1))
