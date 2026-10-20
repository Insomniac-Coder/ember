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
            for item in bag.get_mut():
                _ = bag.get_mut().put(item)    #$ error[E3020]: cannot mutate `*bag.pointer` while it is borrowed by this loop
                println(bag.get().size())    #$ error[E3021]: is borrowed here and mutably borrowed elsewhere
        Err(error):
            println(error.what)
    row = cpp.Row(cells = [5, 6, 7])
    for cell in row:
        row.cells[0] = cell    #$ error[E3020]: cannot mutate `row` while it is borrowed by this loop
