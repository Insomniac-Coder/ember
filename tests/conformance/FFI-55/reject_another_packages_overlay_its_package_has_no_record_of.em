#$ test: compile-fail
#$ rules: FFI-55, FFI-30c
# `[FFI-55]` — another package's overlay that its package's `ember.overlays` does not record is
# `E5010`: nothing says which header it was checked against.

import c "lib.h" with (overlay = "dep/overlays/unrecorded.em")    #$ error[E5010]: `dep/overlays/unrecorded.em` is another package's overlay, and that package's `ember.overlays` has no record of the header it was checked against

fn main():
    pass
