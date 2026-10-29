#$ test: run-pass
#$ rules: FFI-17a, FFI-17e, WK-14
#$ warning[L3015]: `shared_counter` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `watch` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Counter.value` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `owners` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `destroyed` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 9 false 2
#$ stdout: true true true 1
# `[FFI-17a]`, `[WK-14]` — a `std::weak_ptr<T>` is a `CppWeak[T]`, never a `Weak`: `lock()` gives
# a `CppShared[T]` of the object while C++'s count keeps it (a second owner, which C++ counts), and
# `None` once the last `std::shared_ptr` has gone, as `expired()` says; a copy (`clone()`) observes
# the same object. `[FFI-17e]` — `std.ffi`'s.

from std.ffi import CppShared, CppWeak
import cpp "library.hpp"

# The weak pointer outlives every owner of the counter: they end with this call.
fn watched() -> CppWeak[cpp.Counter]:
    unsafe:  # SAFETY(ffi): calls with values the bridge types own
        shared = cpp.shared_counter(9).unwrap()
        weak = cpp.watch(shared)
        match weak.lock():
            Some(locked):
                println(locked.get().value(), weak.expired(), cpp.owners(locked))
            None:
                println("gone")
        return cpp.watch(shared)

fn main():
    weak = watched()
    copy = weak.clone()
    gone = weak.lock().is_none()
    unsafe:  # SAFETY(ffi): a scalar call
        println(weak.expired(), copy.expired(), gone, cpp.destroyed())
