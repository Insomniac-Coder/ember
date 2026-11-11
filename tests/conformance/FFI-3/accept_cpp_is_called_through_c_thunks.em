#$ test: run-pass
#$ rules: FFI-3
#$ profiles: debug, development, release
#$ stdout: 21
#$ 8
# `[FFI-3]` — Ember never links a C++ mangled symbol: the importer generates `extern "C"`
# thunks compiled by the project's own C++ compiler, and Ember calls those. `triple` is `inline`
# (it has no symbol to link at all) and `bump` is a member: both work only through a thunk.

from std.ffi import c_int
import cpp "thunk.hpp"

fn main():
    n: c_int = 0
    unsafe:  # SAFETY(ffi): `triple` has no precondition
        n = cpp.triple(7)
#$ warning[L3015]: `triple` is a C++ function the program calls in unsafe code
    println(n)
    c = cpp.Counter(7)
    m: c_int = 0
    unsafe:  # SAFETY(ffi): `bump` has no precondition
        m = c.bump()
#$ warning[L3015]: `Counter.bump` is a C++ function the program calls in unsafe code
    println(m)
