#$ test: compile-fail
#$ rules: FFI-55, FFI-30c
# `[FFI-55]` — another package's overlay that records no header hash is `E5010` too: nothing says it was
# written for the header imported here. Its help gives the line that records it.
#$ help: with (header_hash = "blake3:

import c "lib.h" with (overlay = "dep/overlays/unhashed.em")    #$ error[E5010]: `dep/overlays/unhashed.em` is another package's overlay and records no `header_hash`, so nothing says it was written for the `lib.h` imported here

fn main():
    pass
