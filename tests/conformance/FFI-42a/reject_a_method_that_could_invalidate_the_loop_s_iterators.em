#$ test: compile-fail
#$ rules: FFI-42a
#$ warning[W5002]: `Bag.begin` is not imported from `containers.hpp`: its result
#$ warning[W5002]: `Bag.end` is not imported from `containers.hpp`: its result
# `[FFI-42a]` — iterating an imported C++ container borrows it mutably for the loop, so a method
# that could invalidate its iterators — `put`, a `mut self` method, which may grow the vector the
# loop walks — is refused inside the loop by the ordinary borrow rules (`E3020`), as is any other
# use of the container (`E3021`); a by-value class's field is refused alike.

import cpp "containers.hpp" with (overlay = "overlays/containers.em")

fn main():
    match cpp.Bag.new():
        Ok(bag):
            for item in bag:
                _ = bag.put(item)    #$ error[E3020]: `_ = bag.put(item)` (line 16) changes `bag`, which this loop walks (line 15); change it after the loop: move line 16, `_ = bag.put(item)`, below the loop
                println(bag.size())    #$ error[E3021]: is borrowed here and mutably borrowed elsewhere
        Err(error):
            println(error.what)
    row = cpp.Row(cells = [5, 6, 7])
    for cell in row:
        row.cells[0] = cell    #$ error[E3020]: `row.cells[0] = cell` (line 22) changes `row`, which this loop walks (line 21); change it after the loop: move line 22, `row.cells[0] = cell`, below the loop
