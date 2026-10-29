#$ test: compile-fail
#$ rules: FFI-11f
#$ warning[L3015]: `Shelf.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `numbers` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `pick` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Shelf.last_of` is a C++ function the program calls in unsafe code
#$ warning[W5001]: macro `LIFETIMEBOUND` is not imported from `lifetimes.hpp`
# `[FFI-11f]` — the lifetime `[[clang::lifetimebound]]` states is enforced on the Ember side: `pick`'s
# result borrows `values` and `last_of`'s the other shelf (not only the receiver, as a method's
# `T&` would otherwise), so neither result outlives what it borrows.

from std.ffi import CppError, CppVector, ForeignBox, c_int
import cpp "lifetimes.hpp"

fn shelf() -> Result[ForeignBox[cpp.Shelf], CppError]:
    unsafe:  # SAFETY(ffi): the constructor makes a shelf the box owns
        return cpp.Shelf.new()

fn second_of_new() -> ref c_int:
    unsafe:  # SAFETY(ffi): a vector of three, read while it lives
        match cpp.numbers():
            Ok(values):
                return cpp.pick(values, 1)    #$ error[E3060]: `values` does not live long enough
            Err(_):
                panic("no numbers")

fn last_of_new(mine: ref cpp.Shelf) -> ref c_int:
    match shelf():
        Ok(theirs):
            unsafe:  # SAFETY(ffi): both shelves are live during the call
                return mine.last_of(theirs.get())    #$ error[E3060]: `theirs` does not live long enough
        Err(_):
            panic("no shelf")

fn main():
    println(second_of_new())
