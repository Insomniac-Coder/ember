#$ test: compile-fail
#$ rules: FFI-17e
#$ warning[L3015]: `squares` is a C++ function the program calls in unsafe code
# `[FFI-17e]` — a span obtained from a `CppVector` borrows it, so it cannot outlive it.

from std.ffi import c_int, CppVector
import cpp "library.hpp"

fn first_squares() -> Span[c_int]:
    unsafe:  # SAFETY(ffi): a call with a value
        match cpp.squares(3):
            Ok(values):
                return values.span()    #$ error[E3060]: `values` does not live long enough
            Err(_):
                panic("no squares")

fn main():
    println(first_squares().len())
