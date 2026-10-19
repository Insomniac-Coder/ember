#$ test: compile-fail
#$ rules: FFI-12
#$ error[E5010]: this overlay is for `counter.h`, and the import is of `tally.h`
# `[FFI-12]` — an overlay describes one header: one written for another is `E5010`.

import c "tally.h" with (overlay = "other_header_overlay.em")

fn main():
    pass
