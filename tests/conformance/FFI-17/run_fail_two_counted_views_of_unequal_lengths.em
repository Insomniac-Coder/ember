#$ test: run-fail
#$ rules: FFI-17, FFI-11
#$ profiles: debug
#$ panics: FFI span lengths differ
# `[FFI-11]` (D-808) — two views whose pointers share one length (`dot(left: borrowed count(count),
# right: borrowed count(count), count)`, a C++ function that may throw) must be as long as each
# other: the call panics before C++ runs, as a C function's does.

from std.ffi import c_int
import cpp "texts.hpp" with (overlay = "overlays/texts_dot.em")

fn main():
    left: Array[c_int] = [1, 2, 3]
    right: Array[c_int] = [4, 5]
    match cpp.dot(left.as_span(), right.as_span()):
        Ok(sum):
            println(sum)
        Err(error):
            println(error.what)
