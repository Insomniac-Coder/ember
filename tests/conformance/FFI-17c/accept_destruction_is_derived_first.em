#$ test: run-pass
#$ rules: FFI-17c, CLS-6
#$ profiles: debug, development, release
#$ stdout: 123
# `[FFI-17c]` — an Ember class deriving from a C++ class is destroyed derived-first, as `[CLS-6]`
# says of every class: its `drop` (1), then its fields (2), and only then the C++ base's
# destructor (3), which the C++ side records.

import cpp "nodes.hpp" with (overlay = "overlays/nodes.em")

struct Marker:
    seen: i32 = 0

    fn drop(mut self):
        unsafe:  # SAFETY(ffi): a scalar call
            cpp.record(2)

class Leaf(cpp.Node):
    marker: Marker = Marker()

    fn drop(mut self):
        unsafe:  # SAFETY(ffi): a scalar call
            cpp.record(1)

fn grow():
    leaf = Leaf()
    _ = leaf.marker.seen

fn main():
    grow()
    unsafe:  # SAFETY(ffi): a scalar call
        println(cpp.recorded())
