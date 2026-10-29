#$ test: compile-fail
#$ rules: FFI-55, FFI-30c
# `[FFI-55]` — another package's overlay is checked against its package's `ember.overlays`: a
# declaration it describes that the header imported here declares otherwise is `E5010`, naming the
# declaration as declared here, with the way out.
#$ help: is another package's overlay: that package checks it against this header and records it again

import c "lib.h" with (overlay = "dep/overlays/stale.em")    #$ error[E5010]: `lib_twice` changed since `dep/overlays/stale.em` was checked: the `lib.h` imported here declares `int lib_twice(int)`

fn main():
    pass
