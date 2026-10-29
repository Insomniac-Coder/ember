#$ test: run-pass
#$ rules: FFI-39e
#$ warning[L3015]: `run` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 1 5
# `[FFI-39e]` — the boundary aborts only when an exception or a panic would cross it: C++ calling
# the base implementation and an override, and Ember calling them, return as usual.

import cpp "tasks.hpp" with (overlay = "overlays/tasks.em")

class Plain(cpp.Task):
    pass

class Busy(cpp.Task):
    override fn step(self) -> i32:
        return 5

fn main():
    plain = Plain()
    busy = Busy()
    unsafe:  # SAFETY(ffi): C++ calls each object's virtual while it lives
        println(cpp.run(plain), cpp.run(busy))
