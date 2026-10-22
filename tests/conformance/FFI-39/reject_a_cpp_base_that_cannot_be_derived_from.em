#$ test: compile-fail
#$ rules: FFI-39, FFI-17d
# `[FFI-39]`, `[FFI-17d]` (ODR-221) — a C++ class an Ember class derives from gets a trampoline;
# one that cannot have one is refused where the class names it, saying why: here a base with no
# virtual destructor, through which C++ could not destroy the subclass (`E5059`).

import cpp "panels.hpp"

class Deep(cpp.Flat):    #$ error[E5059]: `Deep` cannot derive from `Flat` of `panels.hpp`: `Flat` has no virtual destructor
    override fn depth(self) -> i32:
        return 3

fn main():
    pass
