#$ test: compile-fail
#$ rules: FFI-17, FFI-11
#$ profiles: debug
#$ error[E5002]: `count(total)` needs integer ABI witness `total`
# `[FFI-11]` (D-808) — a method's `count(n)` names the parameter that holds its length, as a C
# function's does: `total` is no parameter of `Tally::add`, so the entry asserts what it cannot
# (`E5002`), and its call is not made safe. `[IDE-3]` — `main`, which no error is in, is checked on.

import cpp "texts.hpp" with (overlay = "overlays/texts_unmatched.em")

fn main():
    tally = cpp.Tally.new()  #$ warning[L1001]: `tally` is never read
