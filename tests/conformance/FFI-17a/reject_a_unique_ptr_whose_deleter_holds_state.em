#$ test: compile-fail
#$ rules: FFI-17a
#$ warning[W5002]: `Recycler.operator()` is not imported from `deleters.hpp`
#$ warning[W5002]: `make_tracked` is not imported from `deleters.hpp`: its result: `std::unique_ptr<Widget, void (*)(Widget *)>`: a deleter that holds state
# `[FFI-17a]` (ODR-251) — a `std::unique_ptr` whose deleter holds state (here a function pointer) is
# refused: a `ForeignBox` keeps only its object's address, so its state would have nowhere to live.
# The import skips it, saying why (`W5002`), and a use of it is `E5034` (unsupported C++ construct).

import cpp "deleters.hpp"

fn main():
    w = cpp.make_tracked(3)    #$ error[E5034]: `make_tracked` was not imported from `deleters.hpp`: its result
