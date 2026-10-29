#$ test: run-pass
#$ rules: FFI-55, FFI-30c
#$ profiles: debug
#$ stdout: 42
# `[FFI-55]` — another package's overlay (`dep/` here is a package of its own, with its manifest) is
# applied to the header it was checked against: that package's `ember.overlays` records each
# declaration the overlay describes, and the header imported here declares them so.

import c "lib.h" with (overlay = "dep/overlays/matching.em")

fn main():
    println(c.lib_twice(21))
