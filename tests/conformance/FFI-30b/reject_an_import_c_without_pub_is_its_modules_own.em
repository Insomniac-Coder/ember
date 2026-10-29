#$ test: compile-fail
#$ rules: FFI-30b
#$ warning[L3015]: `geom_sum` is a C function the program calls in unsafe code
# `[FFI-30b]` — without `pub`, a module's `import c` binds the header's module for itself alone.

import vendor.private

fn main():
    println(private.origin_sum())
    p = private.geom.Point(x=1, y=2)  #$ error[E1010]: `private` is a module, and what follows it is not one of its items
