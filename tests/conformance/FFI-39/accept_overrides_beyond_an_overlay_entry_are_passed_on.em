#$ test: run-pass
#$ rules: FFI-39
#$ profiles: debug, development, release
#$ stdout: 4205 5
# `[FFI-39]` (ODR-221) — an overlay's `@ffi(trampoline, virtuals=[…])` entry is still accepted, and
# nothing depends on it: a virtual the program's class overrides that the entry does not name
# (`label`, `weight`) is passed on too.

import cpp "stages.hpp" with (overlay = "overlays/stages.em")

class Counting(cpp.Stage):
    ticks: i32 = 0

    override fn tick(self, dt: i32):
        self.ticks += dt

    override fn label(self) -> i32:
        return 42

    override fn weight(self) -> i32:
        return 5

fn main():
    counting = Counting()
    unsafe:  # SAFETY(ffi): a call on an object that lives for it
        println(cpp.run(counting), counting.ticks)
