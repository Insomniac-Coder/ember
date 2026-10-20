#$ test: run-pass
#$ rules: FFI-36, FFI-32
#$ profiles: debug, development, release
#$ stdout: 4 0
#$ stdout: 1
# `[FFI-36]` — a C++ function returning a raw pointer with no ownership contract yields `*mut T`;
# `adopt[T](p)` (`unsafe`) turns it into a `ForeignBox[T]` using the destructor the class's
# declaration names (`@ffi(destroys(…))`, the importer's thunk for its destructor), which frees it
# when the box ends.

from std.ffi import c_int, adopt
import cpp "widgets.hpp"

fn use_one() -> (c_int, c_int):
    unsafe:  # SAFETY(ffi): `make_widget` returns a new object nothing else owns
        widget = adopt[cpp.Widget](cpp.make_widget(4))
        return (widget.get().size(), cpp.Widget.freed().read())

fn main():
    (size, freed) = use_one()
    println(size, freed)
    unsafe:  # SAFETY(ffi): a static counter's address, read once
        println(cpp.Widget.freed().read())
