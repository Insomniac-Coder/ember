#$ test: run-pass
#$ rules: FFI-55, FFI-30c
#$ profiles: debug
#$ stdout: 42
# `[FFI-55]` — another package's overlay (`dep/` here is a package of its own, with its manifest) is
# applied to the header it was written for: it records that header's hash, and the header imported
# here has it.

import c "lib.h" with (overlay = "dep/overlays/matching.em")

fn main():
    println(c.lib_twice(21))
