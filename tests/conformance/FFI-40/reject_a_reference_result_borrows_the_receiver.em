#$ test: compile-fail
#$ rules: FFI-40, LT-1
#$ warning[W5033]: `Bag.take` is not imported from `bag.hpp`
# `[FFI-40]` — a method's `T&` result is a `ref` borrowing its receiver (`[LT-1]`): while it is live,
# the object is borrowed, so reading it another way is refused.

from std.ffi import c_int
import cpp "bag.hpp"

fn main():
    unsafe:  # SAFETY(ffi): the box owns the bag for every call
        bag = cpp.Bag.new()
        slot: ref mut c_int = bag.get_mut().slot()
        slot = bag.get().total() + 1  #$ error[E3021]: `*bag.pointer` is borrowed here and mutably borrowed elsewhere
