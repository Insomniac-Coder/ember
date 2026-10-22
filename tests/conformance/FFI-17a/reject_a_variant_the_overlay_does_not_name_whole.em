#$ test: compile-fail
#$ rules: FFI-17a
#$ error[E5010]: `Pair` has 2 alternatives, and `@ffi(variant=[…])` names 1
#$ error[E5010]: its alternative `std::basic_string<char>` does not map by value
#$ warning[W5002]: `Pair` is not imported from `unnamed_variants.hpp`
#$ warning[W5002]: `Texty` is not imported from `unnamed_variants.hpp`
#$ warning[W5002]: `first` is not imported from `unnamed_variants.hpp`
#$ warning[W5002]: `second` is not imported from `unnamed_variants.hpp`
# `[FFI-17a]` (ODR-198) — a variant crosses as an enum only when the overlay names each
# alternative and each maps by value: otherwise its entry is `E5010`, saying which, and what
# takes it is not imported.

import cpp "unnamed_variants.hpp" with (overlay = "overlays/unnamed_variants.em")

fn main():
    pass
