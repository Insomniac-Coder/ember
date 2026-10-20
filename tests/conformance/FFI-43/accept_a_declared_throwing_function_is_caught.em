#$ test: run-pass
#$ rules: FFI-43, FFI-24
#$ profiles: debug
#$ stdout: 3
# `[FFI-43]`, `[FFI-24]` — `@ffi(noexcept=false)` declares a function throwing: its calls are caught,
# and on a function the header declares non-throwing (`throw()`), the header's word stands, so
# `stated` still returns its `int`.

from std.ffi import c_int
import cpp "policies.hpp" with (overlay = "overlays/throwing.em")

fn main():
    value: c_int = cpp.stated(0)
    println(value)
