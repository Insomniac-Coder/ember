#$ test: run-pass
#$ rules: FFI-41, STA-1
#$ warning[L3015]: `has_last_grid` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ warning[W5034]: `hidden` is not imported from `space.hpp`: it is in an anonymous namespace
#$ stdout: false false
# `[FFI-41]`, the owner's ruling of 2026-10-21 (ODR-163) — a C++ variable holding a pointer
# (`inline Grid* last_grid`) imports as any other: read and written as a global inside `unsafe`,
# and `E7002` does not refuse it, so the header imports whole.

import cpp "space.hpp"

fn main():
    unsafe:  # SAFETY(ffi): a read and a write of a C++ variable, on one thread
        before = cpp.engine.has_last_grid()
        g = cpp.engine.last_grid
        cpp.engine.last_grid = g
        println(before, cpp.engine.has_last_grid())
