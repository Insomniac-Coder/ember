#$ test: compile-fail
#$ rules: FFI-32, FFI-17
#$ warning[L3015]: `twice` is a C++ function the program calls in unsafe code
#$ warning[W5002]: `length` is not imported from `unmappable.hpp`
# `[FFI-32]`, `[FFI-17]` — a function with a parameter that maps to nothing (a pointer to a
# member function) is not imported, and naming it is `E5031`, saying which parameter and why.

import cpp "unmappable.hpp"

fn main():
    unsafe:  # SAFETY(ffi): never run
        println(cpp.twice(2))
        println(cpp.length(3))    #$ error[E5031]: parameter `text`
