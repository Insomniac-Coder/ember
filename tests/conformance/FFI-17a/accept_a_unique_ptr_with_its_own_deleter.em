#$ test: run-pass
#$ rules: FFI-17a
#$ profiles: debug, development, release
#$ warning[W5002]: `Recycler.operator()` is not imported from `deleters.hpp`
#$ warning[W5002]: `make_tracked` is not imported from `deleters.hpp`: its result: `std::unique_ptr<Widget, void (*)(Widget *)>`: a deleter that holds state
#$ stdout: 7 1
#$ stdout: 8 2
# `[FFI-17a]` — a `std::unique_ptr<T, D>` crosses as a `ForeignBox[T]` dropped through its deleter:
# the box's end hands the object to a `D`, as the `std::unique_ptr` would have, and a parameter
# taking one gives the object up. A deleter that holds state (a function pointer, a class with
# members) is not imported: a `ForeignBox` keeps only its object's address.

import cpp "deleters.hpp"

fn made_and_dropped() -> i32:
    unsafe:  # SAFETY(ffi): the widget lives while its value is read
        widget = cpp.make_widget(7)
        return widget.get().value()

fn main():
    seven = made_and_dropped()
    unsafe:  # SAFETY(ffi): a scalar call; the box is given up to the next
        println(seven, cpp.recycled_count())
        given = cpp.make_widget(8)
        println(cpp.take_widget(given), cpp.recycled_count())
