#$ test: compile-fail
#$ rules: FFI-17d
# `[FFI-17d]` — a trampoline's base has a virtual destructor: C++ that deletes the object through
# it must reach the subclass's (`E5059`).

import cpp "panels.hpp" with (overlay = "overlays/flat.em")    #$ error[E5059]: `Flat` has no virtual destructor

fn main():
    pass
