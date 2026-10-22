#$ test: compile-fail
#$ rules: FFI-39
#$ warning[W5002]: an overload
# `[FFI-39]` (ODR-221) — a C++ class with a pure virtual is an abstract class: an Ember class
# deriving from it that does not implement it is refused, as for any abstract class.

import cpp "stages.hpp"

class Blank(cpp.Shape):    #$ error[E2020]: concrete class `Blank` does not implement abstract method `area`
    override fn sides(self) -> i32:
        return 4

fn main():
    pass
