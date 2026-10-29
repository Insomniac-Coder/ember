#$ test: run-fail
#$ rules: FFI-39e, PAN-1
#$ warning[L3015]: `run` is a C++ function the program calls in unsafe code
#$ panics: no step today
# `[FFI-39e]` — an Ember panic never crosses into C++: an override C++ calls through the trampoline
# panics, and the program aborts there (`[PAN-1]`), with no C++ frame unwound.

import cpp "tasks.hpp" with (overlay = "overlays/tasks.em")

class Lazy(cpp.Task):
    override fn step(self) -> i32:
        panic("no step today")

fn main():
    lazy = Lazy()
    unsafe:  # SAFETY(ffi): C++ calls the object's virtual while it lives
        println(cpp.run(lazy))
