#$ test: compile-fail
#$ rules: FFI-17b
# `[FFI-17b]` — a template `instantiate=[…]` does not list is not imported, and no Ember code,
# generic or not, may instantiate it (`E5055`).

import cpp "templates.hpp"

fn widest[T](a: T, b: T) -> T:
    unsafe:  # SAFETY(ffi): never compiled
        return cpp.biggest(a, b)    #$ error[E5055]: a C++ template

fn main():
    println(widest(1, 2))
