#$ test: compile-fail
#$ rules: FFI-21, FFI-22, FFI-1, FFI-11
#$ profiles: debug
# `[FFI-21]`, `[FFI-22]` (ODR-253) — a C++ function's or method's `nullable` callback, through the
# import's wrapper, which passes its parameter on: what a lambda written in `Some` at the call
# captures is `Send` (`E7004`) unless the contract names a thread, checked where the lambda is
# written. `[FFI-1]`, `[FFI-11]` (ODR-247) — `threads(creator)` and `handle` on a C++ function's
# entry are contract errors (`E5002`), as on a C function; so is (ODR-245) `user_data=right@3` for a
# callback of two parameters.

from std.ffi import c_int
import cpp "hooks.hpp" with (overlay = "overlays/hooks_on_a_function.em")

class Counter:
    count: c_int = 0

fn main():
    counter = Counter()
    println(cpp.maybe_hook(Some(owned fn(value: c_int) -> c_int => counter.count + value), 1))    #$ error[E7004]: captures `counter`, a `Counter`, which is not Send
    panel = cpp.Panel.new()
    println(panel.get().on_resize(Some(owned fn(size: c_int) -> c_int => counter.count + size), 2))    #$ error[E7004]: captures `counter`, a `Counter`, which is not Send
    println(cpp.maybe_hook(None, 1), panel.get().on_resize(Some(fn(size: c_int) -> c_int => size), 2))
#$ error[E5002]: `threads(creator)` is a word of a type's or a result's contract, not a function's
#$ error[E5002]: `handle` is a word of a type's contract, not a function's
#$ error[E5002]: parameter 3 of the callback `pick` does not exist: it takes 2
