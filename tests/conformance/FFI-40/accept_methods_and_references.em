#$ test: run-pass
#$ rules: FFI-40, FFI-17
#$ profiles: debug, development, release
#$ warning[W5033]: `Bag.take` is not imported from `bag.hpp`
#$ stdout: 14
#$ stdout: 12 0 19
#$ stdout: 20
# `[FFI-40]` — a `const` method takes `self`, any other `mut self`; an `&&`-qualified one is
# skipped (`W5033`). A `T&&` parameter is `owned T` (moved in), a `const T&` a borrowed `T`, a
# `T&` of a type by value `mut T`; of an opaque class, which takes no parameter mode (it is
# incomplete, `E5015`), a `const T&` is a `ref T` and a `T&` a `ref mut T`. A method's `T&`
# result is a `ref mut` borrowing its receiver, and a `const T&` one a `ref` (`[LT-1]`), in a
# `Result` when the method may throw (`[FFI-24]`).

from std.ffi import c_int
import cpp "bag.hpp"

fn main():
    value: c_int = 5
    unsafe:  # SAFETY(ffi): the boxes own the bags for every call
        bag = cpp.Bag.new()
        other = cpp.Bag.new()
        other.get_mut().add(cpp.Item(value = 7))
        bag.get_mut().put(cpp.Item(value = 3))
        bag.get_mut().merge(other.get())
        spare = cpp.Bag.new()
        spare.get_mut().put(cpp.Item(value = 4))
        bag.get_mut().drain(spare.get_mut())
        println(cpp.both(bag.get(), spare.get()))
        slot: ref mut c_int = bag.get_mut().slot()
        slot = other.get().total() + 5
        cpp.reset(value)
        peeked: ref c_int = bag.get().peek()
        println(peeked, value, bag.get().total() + other.get().total())
        match bag.get_mut().checked():
            Ok(total):
                total = 20
            Err(error):
                println(error.what)
        println(bag.get().total())
