#$ test: compile-fail
#$ rules: FFI-20a
#$ warning[W5002]: `Packed` is imported from `unrepresentable.h` only as an opaque type
#$ warning[W5002]: `unrepresentable_hidden` is not imported
#$ warning[W5002]: `unrepresentable_log` is not imported
#$ warning[W5002]: `unrepresentable_legacy` is not imported
#$ warning[W5002]: `unrepresentable_number` is not imported
# `[FFI-20a]` — a reference to a declaration the importer skipped says it was skipped and why,
# never "cannot find".

import c "unrepresentable.h"

fn main():
    unsafe:  # SAFETY(ffi): never compiled
        c.unrepresentable_log(c"%d")  #$ error[E1010]: `unrepresentable_log` was not imported from `unrepresentable.h`: variadic
        x = c.unrepresentable_legacy  #$ error[E1010]: `unrepresentable_legacy` was not imported from `unrepresentable.h`: declared without a prototype
        y = c.unrepresentable_hidden  #$ error[E1010]: `unrepresentable_hidden` was not imported from `unrepresentable.h`: a global with internal linkage
