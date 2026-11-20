# Clause FFI-56.2: "within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise, naming where the
# pointer is held)"
#$ test: compile-fail
#$ rules: FFI-56, FFI-2
#$ error[E5002]
# `log_fn` takes a `const char*`, which `overlays/safe_log.em` cannot assert safe: code only, and the
# error is the overlay's, so the claim is the file's.

import c "hooks.h" with (overlay = "overlays/safe_log.em") as h

fn main():
    pass
