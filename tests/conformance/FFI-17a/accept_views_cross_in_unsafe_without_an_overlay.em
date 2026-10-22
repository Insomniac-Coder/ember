#$ test: run-pass
#$ rules: FFI-17a, FFI-38
#$ profiles: debug, development, release
#$ stdout: 6 -1 3
#$ stdout: 5 7 0
#$ stdout: 10 1
# `[FFI-17a]`, `[FFI-38]` — with no overlay, each call is the caller's to keep (`unsafe`), and the
# standard-library types cross alike: a view parameter is a `Span`, an optional value an `Option`, a
# `std::unique_ptr` the `ForeignBox` given up; but a view result, whose lifetime no header states,
# is its pointer and its length.

from std.ffi import c_int
import cpp "views.hpp" with (project = "views")

fn main():
    values: Array[c_int] = [1, 2, 3]
    unsafe:  # SAFETY(ffi): each view and value lives through its call
        total = cpp.sum(values.as_span())
        cpp.negate(values.as_mut_span())
        println(total, values[0], cpp.vowels("ember burns".as_bytes()))
        words = cpp.Words.new().unwrap()
        _, length = words.get().first()
        println(length, cpp.digit(55).unwrap_or(-1), cpp.or_zero(None))
        box = cpp.make_box(9).unwrap()
        println(cpp.consume(box), cpp.boxes_destroyed())
