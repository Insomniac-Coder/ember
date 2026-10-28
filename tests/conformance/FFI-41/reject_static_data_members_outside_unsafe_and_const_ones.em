#$ test: compile-fail
#$ rules: FFI-41, FFI-10
#$ warning[W5034]: `hidden` is not imported from `space.hpp`: it is in an anonymous namespace
# `[FFI-41]` (ODR-161) — a C++ static data member, and a namespace's variable, is a foreign static
# (`[FFI-10]`): reading or writing one outside `unsafe` is `E5002`, and a `const` one, which is read
# anywhere, is never written (`E2140`). The private static the module declares for a member is not
# a second name for it.

import cpp "space.hpp"

fn main():
    count = cpp.engine.Grid.made  #$ error[E3100]: reading a foreign static requires `unsafe`
    cpp.engine.frames += 1  #$ error[E3100]: writing a foreign static requires `unsafe`
    half = cpp.engine.Grid.scale
    unsafe:  # SAFETY(ffi): a write of a C++ variable
        cpp.engine.Grid.scale = 2.0  #$ error[E2140]: a foreign static without `mut` cannot be assigned
        cpp.engine.Grid_made = 1  #$ error[E1052]
    println(count, half)
