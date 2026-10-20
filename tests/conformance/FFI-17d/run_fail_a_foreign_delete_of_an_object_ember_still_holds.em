#$ test: run-fail
#$ rules: FFI-17d
#$ panics: a foreign `delete` of a
# `[FFI-17d]` — a foreign `delete` destroys an `owner="foreign"` object only when the count C++ was
# given is its last: Ember code still holding a handle would be left with a deleted C++ object, so
# it panics.

import cpp "nodes.hpp" with (overlay = "overlays/foreign.em")

class Leaf(cpp.Node):
    override fn value(self) -> i32:
        return 9

fn main():
    leaf = Leaf()
    cpp.keep(leaf)
    unsafe:  # SAFETY(ffi): a scalar call
        cpp.release()
    println(leaf.value())
