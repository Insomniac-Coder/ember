#$ test: run-pass
#$ rules: FFI-36, FFI-17a
#$ warning[W5002]: `Recycler.operator()` is not imported from `gizmos.hpp`
#$ warning[L3015]: `raw_gizmo` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Gizmo.value` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `recycled_count` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `make_gizmo` is a C++ function the program calls in unsafe code
#$ stdout: 3 1
#$ stdout: 5
#$ stdout: 2
# `[FFI-36]` — with no `@ffi(destroys(f))`, `adopt` ends a C++ class's object through the deleter
# `D` that the `std::unique_ptr<T, D>`s the import gives name (`[FFI-17a]`): `Recycler`, the same
# one a `std::unique_ptr<Gizmo, Recycler>` result's end calls.

from std.ffi import adopt
import cpp "gizmos.hpp"

fn adopted() -> i32:
    # SAFETY(ffi): `raw_gizmo` gives a new object, made with `new`, that nothing else owns
    unsafe:
        gizmo = adopt[cpp.Gizmo](cpp.raw_gizmo(3))
        return gizmo.value()

fn main():
    value = adopted()
    # SAFETY(ffi): a counter's read, and a box handed back to Ember
    unsafe:
        println(value, cpp.recycled_count())
        boxed = cpp.make_gizmo(5)
        println(boxed.value())
    # SAFETY(ffi): a counter's read
    unsafe:
        println(cpp.recycled_count())
