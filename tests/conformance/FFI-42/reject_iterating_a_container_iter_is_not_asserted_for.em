#$ test: compile-fail
#$ rules: FFI-42
#$ warning[W5002]: `Bag.begin` is not imported from `containers.hpp`: its result
#$ warning[W5002]: `Bag.end` is not imported from `containers.hpp`: its result
# `[FFI-42]` — `iter()`, which a `for` loop over a C++ container calls, is a call into C++ like any
# method: with no overlay entry asserting it safe (`[FFI-2]`), the loop needs `unsafe`.

import cpp "containers.hpp"

fn main():
    row = cpp.Row(cells = [5, 6, 7])
    for cell in row:    #$ error[E3100]: `row` (line 12) calls `iter`, an `unsafe fn` whose callers must make true what its `@safety` says
        println(cell)
