#$ test: run-pass
#$ rules: FFI-30, FFI-30a
#$ profiles: debug, development, release
#$ stdout: 3 25
#$ stdout: 8 true
# `[FFI-30]` (D-879) — one C++ header imported by two modules, each spelling its path its own way
# (`"classes.hpp"` here, `"../classes.hpp"` from `helpers/`): with the same overlay the two imports
# are one module, whichever spelling; with another overlay they are two modules, and the class
# `Counter` is one entity, its members written once, by the import that wrote the class, so each
# module reaches the same `new`, `add` and `count`. Each module's own functions stay its own.

import helpers.counted
import cpp "classes.hpp" with (overlay = "overlays/classes.em")

fn main():
    counter = cpp.Counter.new()
    counter.get_mut().add(3)
    println(counter.get().count(), counted.made())
    println(counted.twice(4), cpp.doubled(4) == counted.twice(4))
