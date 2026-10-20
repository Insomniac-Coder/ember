#$ test: compile-fail
#$ rules: FFI-48
#$ warning[W5002]: `Both` is not imported from `shapes.hpp`: multiple inheritance
#$ warning[W5002]: `Shared` is not imported from `shapes.hpp`: multiple inheritance, or a virtual base
# `[FFI-48]` — multiple inheritance and virtual bases are unsupported: the pointer adjustment is
# ABI-private and differs between MSVC and Itanium, so such a class is not imported, and naming it
# is `E5034`, naming the row; the header's other declarations import as ever.

import cpp "shapes.hpp"

fn main():
    unsafe:  # SAFETY(ffi): never run
        println(cpp.answer())
        both = cpp.Both.new()    #$ error[E5034]: multiple inheritance
        shared = cpp.Shared.new()    #$ error[E5034]: a virtual base
