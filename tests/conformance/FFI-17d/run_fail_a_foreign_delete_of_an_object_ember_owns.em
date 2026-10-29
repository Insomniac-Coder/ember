#$ test: run-fail
#$ rules: FFI-17d
#$ warning[L3015]: `keep` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `release` is a C++ function the program calls in unsafe code
#$ panics: a foreign `delete` of a `Node`, which Ember owns
# `[FFI-17d]` — with `owner="ember"` (the default) the Ember count owns the object: a foreign
# `delete` of it panics rather than destroying what Ember code holds.

import cpp "nodes.hpp" with (overlay = "overlays/ember.em")

class Leaf(cpp.Node):
    override fn value(self) -> i32:
        return 9

fn main():
    leaf = Leaf()
    unsafe:  # SAFETY(ffi): C++ keeps the node only until it deletes it
        cpp.keep(leaf)
        cpp.release()
    println(leaf.value())
