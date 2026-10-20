#$ test: compile-fail
#$ rules: FFI-44
#$ warning[W5001]: macro `SQUARE` is not imported from `rows.hpp`: a function-like macro
#$ warning[W5002]: `call_with_one` is not imported from `rows.hpp`: parameter `f`
#$ warning[W5002]: `Both` is not imported from `rows.hpp`: multiple inheritance
# `[FFI-44]` — what is not automatic is said so where the program names it: a template the import
# does not instantiate stays a native island (`E5055`), as does a class with multiple bases
# (`E5034`); a function-like macro needs an overlay entry; a `std::function` is never a parameter
# (`E5030`); and a pointer whose ownership the header does not state keeps its call unsafe.

import cpp "rows.hpp" with (instantiate = ["largest<int>"])

fn main():
    a = cpp.smallest(1, 2)    #$ error[E5055]: `smallest` was not imported from `rows.hpp`: a C++ template
    b = cpp.Both.new()    #$ error[E5034]: `Both` was not imported from `rows.hpp`: multiple inheritance
    c = cpp.SQUARE(3)    #$ error[E1010]: an overlay's `@ffi(macro_fn)` entry declares its signature
    d = cpp.call_with_one    #$ error[E5030]: a `std::function` is never a parameter type
    e = cpp.made()    #$ error[E3100]: calling an `unsafe fn` requires an `unsafe` block
