#$ test: compile-fail
#$ rules: FFI-39b
# `[FFI-39b]` — a class deriving from a C++ base with no default constructor declares an `init`
# that calls `super.init(…)` (`E5058` otherwise).

import cpp "panels.hpp" with (overlay = "overlays/panels.em")

class Wide(cpp.Panel):    #$ error[E5058]: class `Wide` declares no `init`, and its C++ base `Panel` has no default constructor
    pass

class Narrow(cpp.Panel):
    fn init(self):
        super.init(3)

fn main():
    pass
