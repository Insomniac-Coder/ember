#$ test: compile-fail
#$ rules: FFI-55, FFI-30c
# `[FFI-55]` — another package's overlay records the hash of the header it was written for; applied to
# a header whose hash differs, it is `E5010`, naming the overlay, the header and both hashes, with the
# two ways out.
#$ help: import the header the overlay was written for, or write this package's own overlay for the header it imports

import c "lib.h" with (overlay = "dep/overlays/stale.em")    #$ error[E5010]: `dep/overlays/stale.em` is another package's overlay, written for a `lib.h` whose hash is `blake3:0000000000000000000000000000000000000000000000000000000000000000`; the `lib.h` imported here has hash `blake3:

fn main():
    pass
