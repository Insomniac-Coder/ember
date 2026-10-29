#$ test: compile-fail
#$ rules: FFI-8
#$ warning[L3015]: `format_list` is a C function the program calls in unsafe code
# `[FFI-8]` — a `VaList` is not `Copy`: whoever it is passed to reads it, so it is passed on once.

from std.ffi import VaList, c_char, c_int
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v

extern "C" fn sink(context: *mut void, format: *c_char, owned arguments: VaList):
    unsafe:  # SAFETY(ffi): the second use is the case the checker refuses
        v.format_list(null[*mut c_char](), 0, format, arguments)
        v.format_list(null[*mut c_char](), 0, format, arguments)    #$ error[E3040]

fn main():
    pass
