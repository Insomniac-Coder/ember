#$ test: run-pass
#$ rules: FFI-17d, FFI-17c
#$ warning[L3015]: `record` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `recorded` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `kept_value` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `release` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 0 9
#$ stdout: 123 -1
# `[FFI-17d]` — `@ffi(trampoline, owner="foreign")` lets a foreign `delete` destroy the object: an
# `owned` parameter gives C++ a count of its own (the handle the caller passed goes with it), and
# when C++ deletes the object, the Ember side ends with that count — its `drop` (1), its fields (2),
# then the C++ destructor (3), derived-first as ever (`[FFI-17c]`).

import cpp "nodes.hpp" with (overlay = "overlays/foreign.em")

struct Marker:
    seen: i32 = 0

    fn drop(mut self):
        unsafe:  # SAFETY(ffi): a scalar call
            cpp.record(2)

class Leaf(cpp.Node):
    marker: Marker = Marker()

    override fn value(self) -> i32:
        return 9

    fn drop(mut self):
        unsafe:  # SAFETY(ffi): a scalar call
            cpp.record(1)

fn hand_over():
    cpp.keep(Leaf())

fn main():
    hand_over()
    unsafe:  # SAFETY(ffi): scalar calls; C++ holds the node it was given
        println(cpp.recorded(), cpp.kept_value())
        cpp.release()
        println(cpp.recorded(), cpp.kept_value())
