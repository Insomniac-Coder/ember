#$ test: compile-fail
#$ rules: FFI-20a
#$ warning[W5002]: `Number` is imported from `unrepresentable.h` only as an opaque type
#$ warning[W5002]: `Flags` is imported from `unrepresentable.h` only as an opaque type
#$ warning[W5002]: `unrepresentable_hidden` is not imported
#$ warning[W5002]: `unrepresentable_log` is not imported
#$ warning[W5002]: `unrepresentable_precise` is not imported
#$ warning[W5002]: `unrepresentable_number` is not imported
# `[FFI-20a]` — a reference to a declaration the importer skipped says it was skipped and why,
# never "cannot find".

import c "unrepresentable.h"

fn main():
    unsafe:  # SAFETY(ffi): never compiled
        c.unrepresentable_log(c"%d")  #$ error[E1010]: `unrepresentable_log` was not imported from `unrepresentable.h`: variadic
        x = c.unrepresentable_precise  #$ error[E1010]: `unrepresentable_precise` was not imported from `unrepresentable.h`: its result: `long double`
        y = c.unrepresentable_hidden  #$ error[E1010]: `unrepresentable_hidden` was not imported from `unrepresentable.h`: a global with internal linkage
