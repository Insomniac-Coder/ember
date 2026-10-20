#$ test: compile-fail
#$ rules: FFI-43, FFI-24
# `[FFI-43]` — a function whose exception policy cannot be derived and is not declared is `E5061`
# where it is used. Here the project is freestanding (`-nostdinc`), so Clang cannot evaluate the
# `noexcept(expr)` of `add` or `sub`; the overlay declares `sub`'s, and `sub` is imported.

import cpp "freestanding.hpp" with (project = "freestanding", overlay = "overlays/freestanding.em")

fn main():
    println(cpp.sub(3, 1))
    unsafe:  # SAFETY(ffi): a scalar call
        println(cpp.add(1, 2))    #$ error[E5061]: `add` was not imported from `freestanding.hpp`: its exception policy cannot be derived
