#$ test: run-pass
#$ rules: FFI-17e
#$ warning[L3015]: `shared_counter` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `destroyed` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `owners` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 0 1
#$ stdout: 1
# `[FFI-17e]` — the bridge types are `std.ffi`'s, and their values end through C++: a `CppShared`
# copies its `std::shared_ptr` when cloned and destroys it when dropped, so the object it shares
# is destroyed, by C++, when the last one ends — not when the first does.

from std.ffi import CppShared
from std.mem import drop
import cpp "library.hpp"

fn share():
    unsafe:  # SAFETY(ffi): calls with values the bridge types own
        match cpp.shared_counter(1):
            Ok(first):
                second = first.clone()
                drop(first)
                println(cpp.destroyed(), cpp.owners(second))
            Err(error):
                println(error.what)

fn main():
    share()
    unsafe:  # SAFETY(ffi): a scalar call
        println(cpp.destroyed())
