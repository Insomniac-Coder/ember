#$ test: run-pass
#$ rules: FFI-17a, STD-14, ERR-13
#$ profiles: debug, development, release
#$ stdout: 7 7
#$ stdout: true
# `[STD-14]`, `[ERR-13]` (ODR-254) — `CppShared.get` is an accessor whose failure, an empty
# `std::shared_ptr`, is the caller's bug: it panics naming its twin, `try_get`, which gives the
# object it shares, or `None` for an empty one.

from std.ffi import CppShared
import cpp "gauges.hpp"

fn reading(shared: CppShared[cpp.Gauge]) -> int:
    match shared.try_get():
        Some(found):
            unsafe:  # SAFETY(ffi): a `const` member call on the object the pointer shares
                return found.reading() as int
        None:
            return -1

fn main():
    unsafe:  # SAFETY(ffi): calls with values the bridge types own
        match cpp.gauge(7):
            Ok(shared):
                println(reading(shared), shared.get().reading())
            Err(error):
                println(error.what)
        println(reading(cpp.no_gauge()) == -1 and cpp.no_gauge().try_get().is_none())
