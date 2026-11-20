# Clause FFI-56.2: "An overlay may assert one safe to call with `@ffi(safe)` on the typedef's entry
# (`@ffi(safe) type PFN_log_message`) when every parameter and result is a scalar or a `@layout(c)`
# value type"
#$ test: compile-pass
#$ rules: FFI-56, TCB-4
# `release_fn` takes an `int`: the overlay asserts it safe, so the call needs no `unsafe:`. The test
# only compiles.

import c "hooks.h" with (overlay = "overlays/safe_release.em") as h

fn main():
    release = h.get_release()
    release(1)
