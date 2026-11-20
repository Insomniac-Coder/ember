# Clause FFI-42a.1: "A `for` loop over an imported C++ container walks it, so nothing changes the
# container while the loop walks it: a call of any method imported as changing its object
# (`[FFI-40]`), the `const` ones `[FFI-40a]` imports so included, is refused inside the loop
# (`E3020`)"
#$ test: compile-fail
#$ rules: FFI-42a, FFI-40, FFI-40a
# Inside the walk, `first` (asserted `@ffi(const)`) reads the row; `bump` and the `const` `looked`,
# whose entry does not say `@ffi(const)`, are refused: code only. The test only compiles.

import cpp "rows.hpp" with (overlay = "overlays/rows.em")

fn main():
    row = cpp.Row.new()
    for cell in row:
        println(cell, row.first())
        row.bump()    #$ error[E3020]
        println(row.looked())    #$ error[E3020]
