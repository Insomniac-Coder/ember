#$ test: run-pass
#$ rules: FFI-17a
#$ warning[L3015]: `make_widget` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Widget.value` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `recycled_count` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `take_widget` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ warning[W5002]: `Recycler.operator()` is not imported from `deleters.hpp`
#$ warning[W5002]: `make_tracked` is not imported from `deleters.hpp`: its result: `std::unique_ptr<Widget, void (*)(Widget *)>`: a deleter that holds state
#$ stdout: 7 1
#$ stdout: 8 2
# `[FFI-17a]` — a `std::unique_ptr<T, D>` crosses as an `owns T` ended through its deleter:
# the box's end hands the object to a `D`, as the `std::unique_ptr` would have, and a parameter
# taking one gives the object up. A deleter that holds state (a function pointer, a class with
# members) is not imported: an `owns T` holds one address. `[FFI-36a]` — the deleter is the
# class's one destructor, as every `std::unique_ptr` of the class gives it.

import cpp "deleters.hpp"

fn made_and_dropped() -> i32:
    unsafe:  # SAFETY(ffi): the widget lives while its value is read
        widget = cpp.make_widget(7)
        return widget.value()

fn main():
    seven = made_and_dropped()
    unsafe:  # SAFETY(ffi): a scalar call; the box is given up to the next
        println(seven, cpp.recycled_count())
        given = cpp.make_widget(8)
        println(cpp.take_widget(owns given), cpp.recycled_count())
