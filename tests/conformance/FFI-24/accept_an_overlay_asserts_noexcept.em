#$ test: run-pass
#$ rules: FFI-24b, FFI-17
#$ profiles: debug, development, release
#$ stdout: 7
# `[FFI-24b]` — an overlay may assert `@ffi(noexcept)` for a function the header leaves throwing:
# `risky` then returns its `int` directly, not a `Result`, and its entry makes it safe to call. `[FFI-17]` — overlays for C++ are
# `overlay cpp "Header.hpp":`, with `[GRM-35]`'s grammar.

from std.ffi import c_int
import cpp "policies.hpp" with (overlay = "overlays/policies.em")

fn main():
    value: c_int = cpp.risky(7)
    println(value)
