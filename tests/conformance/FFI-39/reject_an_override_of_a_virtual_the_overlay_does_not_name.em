#$ test: compile-fail
#$ rules: FFI-39, FFI-48
# `[FFI-39]`, `[FFI-48]` — the trampoline calls Ember only for the virtuals its overlay names: an
# Ember override of another C++ virtual (`unnamed`) would never be called by C++, so it is `E5056`.

import cpp "layers.hpp" with (overlay = "overlays/layers.em")

class Mine(cpp.Layer):
    override fn unnamed(self) -> i32:    #$ error[E5056]: `unnamed` overrides a C++ virtual of `Layer` that its trampoline does not override
        return 8

fn main():
    pass
