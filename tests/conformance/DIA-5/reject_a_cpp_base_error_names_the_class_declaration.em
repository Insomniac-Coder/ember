#$ test: compile-fail
#$ rules: DIA-5, FFI-39b
# `[DIA-5]` for C++: an error about an Ember class deriving from a C++ class (here `E5058`, the base
# has no default constructor and the class declares no `init`) names the header and the C++ class
# declaration, its body elided.

import cpp "gauges.hpp" with (overlay = "overlays/gauges.em")

class Dial(cpp.Gauge):    #$ error[E5058]: declared in the header at gauges.hpp:4: `class Gauge {…}`
    pass

fn main():
    pass
